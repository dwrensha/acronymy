use sandstorm::grain_capnp::{ui_view, ui_session};
use sandstorm::web_session_capnp::{web_session};

use std::collections::hash_map::HashMap;
use capnp::Error;
use capnp::capability::Promise;
use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
use sqlite3;

#[derive(Clone, Copy)]
pub struct UiViewImpl;

impl ui_view::Server for UiViewImpl {
    fn get_view_info(&mut self,
                     _params: ui_view::GetViewInfoParams,
                     _results: ui_view::GetViewInfoResults)
                     -> Promise<(), Error>
    {
        Promise::ok(())
    }

    fn new_session(&mut self,
                   _params: ui_view::NewSessionParams,
                   mut results: ui_view::NewSessionResults)
                   -> Promise<(), Error>
    {
        let client: web_session::Client = match WebSessionImpl::new() {
            Ok(session) => {
                web_session::ToClient::new(session).from_server::<::capnp_rpc::Server>()
            }
            Err(_e) => {
                return Promise::err(Error::failed(format!("failed to create web session")));
            }
        };
        // we need to do this dance to upcast.
        results.get().set_session(ui_session::Client { client : client.client});
        Promise::ok(())
    }
}

pub struct WebSessionImpl {
    db: sqlite3::Database,
}

unsafe impl Send for WebSessionImpl {}

impl WebSessionImpl {
    pub fn new() -> sqlite3::SqliteResult<WebSessionImpl> {
        let mut db = try!(sqlite3::open("/var/data.db"));
        db.set_busy_timeout(1000); // try for a least a second
        Ok(WebSessionImpl {
            db : db,
        })
    }
}

impl ui_session::Server for WebSessionImpl {

}

impl WebSessionImpl {

    fn is_word(&self, word : &str) -> sqlite3::SqliteResult<bool> {

        if ! word.chars().all(|c| c.is_alphanumeric()) { return Ok(false); }

        let mut cursor = try!(self.db.prepare(
            &format!("SELECT * FROM Words WHERE Word = \"{}\";", word),
            &None));

        return Ok(try!(cursor.step_row()).is_some());
    }

    fn validate_def(&self, word : &str, definition : &[&str]) -> sqlite3::SqliteResult<bool> {
        if definition.len() != word.len() { return Ok(false); }
        let mut idx = 0;
        for &d in definition.iter() {
            if !(try!(self.is_word(d)) && d.len() > 0 && d.as_bytes()[0] == word.as_bytes()[idx]) {
                return Ok(false);
            }

            idx += 1;
        }

        return Ok(true);
    }

    fn write_def(&mut self, word : &str, definition : &[&str]) -> sqlite3::SqliteResult<()> {

        let time : i64 = ::time::get_time().sec;
        let mut query = String::new();
        query.push_str(&format!("BEGIN; DELETE FROM Definitions WHERE Definee =\"{}\"; ", word));
        query.push_str("INSERT INTO Definitions(Definee, Idx, Definer) VALUES");
        let mut idx = 0usize;
        for &d in definition.iter() {
            if idx != 0 { query.push_str(","); }
            query.push_str(&format!("(\"{}\", {}, \"{}\")", word, idx, d));
            idx += 1;
        }
        query.push_str(";");
        query.push_str(&format!("DELETE FROM Log WHERE Word=\"{}\";", word));
        query.push_str(&format!("INSERT INTO Log(Word, Timestamp) VALUES(\"{}\",{});", word, time));
        query.push_str("COMMIT;");

        println!("query: {}", query);

        try!(self.db.exec(&query));

        return Ok(());
    }

    fn get_def(&self, word : &str) -> sqlite3::SqliteResult<String> {

        let mut cursor = try!(self.db.prepare(
            &format!("SELECT * FROM Definitions WHERE Definee = \"{}\";", word),
            &None));

        let mut map = HashMap::<isize, String>::new();

        loop {
            match try!(cursor.step_row()) {
                None => break,
                Some(row) => {
                    let definer = match row[&"Definer".to_string()] { sqlite3::BindArg::Text(ref t) => t.clone(), _ => panic!(), };
                    let idx = match row[&"Idx".to_string()] { sqlite3::BindArg::Integer(ref i) => i.clone(), _ => panic!(), };

                    map.insert(idx, definer);
                }
            }
        }

        if map.len() != word.len() {
            return Ok("<div>this word has no definition yet</div>".to_string());
        } else {

            let mut result = String::new();
            result.push_str("<div>");
            for idx in 0..(word.len() as isize) {
                let definer : &str = &map[&idx];
                result.push_str(&format!(" <a href=\"define?word={word}\">{word}</a> ", word=definer));
            }
            result.push_str("</div>");
            return Ok(result);
        }
    }

    fn count_defs(&self) -> sqlite3::SqliteResult<(isize, isize, Vec<String>)> {
        let mut cursor = try!(self.db.prepare("SELECT COUNT(*) FROM Words;", &None));
        assert!(cursor.step() == sqlite3::ResultCode::SQLITE_ROW);
        let num_words = cursor.get_int(0);

        let mut cursor = try!(self.db.prepare("SELECT COUNT(*) FROM Definitions WHERE Idx = 0;", &None));
        assert!(cursor.step() == sqlite3::ResultCode::SQLITE_ROW);
        let defined_words = cursor.get_int(0);

        let mut recent_words = Vec::new();
        let mut cursor = try!(
            self.db.prepare("SELECT Word, Timestamp FROM Log ORDER BY Timestamp DESC LIMIT 5;", &None));
        loop {
            match try!(cursor.step_row()) {
                None => break,
                Some(row) => {
                    let word : String = match row[&"Word".to_string()] {sqlite3::BindArg::Text(ref t) => t.clone(), _ => panic!(),};
                    recent_words.push(word);
                }
            }
        }
        Ok((defined_words, num_words, recent_words))

    }


    fn construct_page_data(&mut self, path : Vec<String>, query: Option<String>) -> sqlite3::SqliteResult<PageData> {
        if path.len() == 1 && path[0] == "define" {

            let mut query_map = HashMap::<String, String>::new();
            match query {
                None => {}
                Some(q) => {
                    for (k, v) in ::url::form_urlencoded::parse(q.as_bytes()).into_owned() {
                        query_map.insert(k, v);
                    }
                }
            }

            let word : String = match query_map.get(&"word".to_string()) {
                Some(w) if try!(self.is_word(&w)) => {
                    w.clone()
                }
                _ => {
                    return Ok(PageData::Error("that's not a word".to_string()))
                }
            };
            match query_map.get(&"definition".to_string()) {
                None => {
                    let def_div = try!(self.get_def(&word));

                    return Ok(PageData::WordAndDef(word, def_div, None));
                }
                Some(def_query) => {

                    let definition : Vec<&str> = def_query.split(' ').collect();

                    if try!(self.validate_def(&word, &definition)) {

                        try!(self.write_def(&word, &definition));
                        let def_div = try!(self.get_def(&word));
                        return Ok(PageData::WordAndDef(word,
                                                       def_div,
                                                       None));
                    } else {
                        let def_div = try!(self.get_def(&word));
                        return Ok(PageData::WordAndDef(word, def_div, Some("invalid definition".to_string())))
                    }
                }
            }

        } else {
            let (num_defined, total, recent) = try!(self.count_defs());
            return Ok(PageData::HomePage(num_defined, total, recent));
        }
    }

}

const MAIN_CSS : &'static str =
    "body { font-family: Helvetica, Sans, Arial;
            font-size: medium;
             margin-left: auto;
             margin-right: auto;
             text-align: center;
     }
     div {
          padding-bottom: 10pt;
     }
    .word {
        text-align: center;
        font-size: 500%;
     }
     .err {
       font-size: 90%;
       color: #AA0000;
     }
     .title {
       text-align: center;
       font-size:500%;
     }
     ";


const HEADER : &'static str =
  r#"<head><title> acronymy </title><link rel="stylesheet" type="text/css" href="main.css" >
 <meta http-equiv="Content-Type" content="text/html;charset=utf-8" >
  </head>"#;


const LOOKUP_FORM : &'static str =
      r#"<form action="define" method="get">
          <input name="word" maxlength="100"/><button>find word</button></form>"#;

fn define_form(word :&str) -> String {
       format!("<form action=\"define\" method=\"get\">
               <input name=\"word\" value=\"{word}\" type=\"hidden\"/>
               <input name=\"definition\" maxlength=\"2000\"/>
                   <button>submit definition</button></form>", word=word)
}

enum PageData {
    Error(String),
    WordAndDef(String, String, Option<String>),
    HomePage(isize, isize, Vec<String>),
}

fn construct_html(page_data : PageData) -> String {
    let mut result = String::new();
    result.push_str(&format!("<html>{}<body>", HEADER));

    const HOME_LINK : &'static str = "<a href=\"/\">home</a>";
    match page_data {
        PageData::Error(e) => {
            result.push_str(&format!("<div class=\"err\"> {} </div>", e));
            result.push_str(LOOKUP_FORM);
            result.push_str(HOME_LINK);
        }
        PageData::WordAndDef(word, def_div, err) => {
            result.push_str(&format!("<div class=\"word\">{}</div>", word));
            result.push_str(&def_div);

            match err {
                None => {}
                Some(e) => {
                    result.push_str(&format!("<div class=\"err\">{}</div>", e));
                }
            }

            result.push_str(&define_form(&word));
            result.push_str(HOME_LINK);
        }
        PageData::HomePage(num_defined, total, recent) => {
            result.push_str("<div class=\"title\">Acronymy</div>");
            result.push_str("<div>A user-editable, acronym-only dictionary.</div>");
            result.push_str(&format!("<div>So far, we have defined {} out of {} words.</div>",
                                     num_defined, total));
            if recent.len() > 0 {
                result.push_str("<div>Recently modified words: ");
                let mut idx = 0usize;
                for w in recent.iter() {
                    if idx != 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&format!("<a href=\"/define?word={word}\">{word}</a>", word=*w));
                    idx += 1;
                }
                result.push_str(".</div>");
            }
            result.push_str(LOOKUP_FORM);
        }
    }

    result.push_str("</body></html>");
    result
}

impl web_session::Server for WebSessionImpl {
    fn get(&mut self,
           params: web_session::GetParams,
           mut results: web_session::GetResults)
        -> Promise<(), Error>
    {
        println!("GET");
        {
            let params = pry!(params.get());
            let results = results.get();
            let raw_path = format!("/{}", pry!(params.get_path()));
            let mut content = results.init_content();
            content.set_mime_type("text/html");

            // TODO: there's got to be a cleaner way to do this.
            let (path, query) = ::url::Url::parse("http://example.com").map(|base| {
                base.join(&raw_path).map(|url| {
                    let query = url.query().map(|s| s.to_string());
                    url.path_segments().map(move |segs| {
                        let seq_strings: Vec<String> = segs.map(|seg| seg.to_string()).collect();
                        (seq_strings, query)
                    }).unwrap_or((Vec::new(), None))
                })
            }).unwrap_or(Ok((Vec::new(), None))).unwrap_or((Vec::new(), None));

            println!("path = {:?}, query = {:?}", path, query);

            if raw_path == "/main.css" {
                content.set_mime_type("text/css");
                content.get_body().set_bytes(MAIN_CSS.as_bytes())
            } else {
                let page_data = match self.construct_page_data(path, query) {
                    Err(e) => { PageData::Error(format!("database error: {:?} ({})", e, self.db.get_errmsg())) }
                    Ok(page_data) => { page_data }
                };
                content.get_body().set_bytes(construct_html(page_data).as_bytes());
            }
        }
        Promise::ok(())
    }
}

pub fn main() -> ::capnp::Result<()> {
    use tokio_core::io::Io;
    use ::std::os::unix::io::{FromRawFd, IntoRawFd};

    let args: Vec<String> = ::std::env::args().collect();
    if args.len() == 4 && args[1] == "--init" {
        println!("initializing...");
        let initdb_path = ::std::path::Path::new(&args[2]);
        let proddb_path = ::std::path::Path::new(&args[3]);
        println!("copying database from {} to {}", args[2], args[3]);
        try!(::std::fs::copy(initdb_path, proddb_path));
        println!("success!");
    }

    let mut core = try!(::tokio_core::reactor::Core::new());
    let handle = core.handle();

    let stream: ::std::os::unix::net::UnixStream = unsafe { FromRawFd::from_raw_fd(3) };
    try!(stream.set_nonblocking(true));
    let stream: ::mio_uds::UnixStream = unsafe { FromRawFd::from_raw_fd(stream.into_raw_fd()) };
    let stream = try!(::tokio_core::reactor::PollEvented::new(stream, &handle));

    let (read_half, write_half) = stream.split();

    let network =
        Box::new(twoparty::VatNetwork::new(read_half, write_half,
                                           rpc_twoparty_capnp::Side::Client,
                                           Default::default()));

    let client = ui_view::ToClient::new(UiViewImpl).from_server::<::capnp_rpc::Server>();

    let rpc_system = RpcSystem::new(network, Some(client.client));

    try!(core.run(rpc_system));
    Ok(())
}
