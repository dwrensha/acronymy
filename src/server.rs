use grain_capnp::{PowerboxCapability, UiView, UiSession};
use web_session_capnp::{WebSession};

use collections::hashmap::HashMap;
use capnp::capability::{ClientHook, FromServer};
use capnp::AnyPointer;
use capnp_rpc::rpc::{RpcConnectionState, SturdyRefRestorer};
use capnp_rpc::capability::{LocalClient};

use sqlite3;

pub struct UiViewImpl;

impl PowerboxCapability::Server for UiViewImpl {
    fn get_powerbox_info(&mut self, context : PowerboxCapability::GetPowerboxInfoContext) {
        context.done()
    }
}

impl UiView::Server for UiViewImpl {
    fn get_view_info(&mut self, context : UiView::GetViewInfoContext) {
        context.done()
    }

    fn new_session(&mut self, mut context : UiView::NewSessionContext) {
        println!("asked for a new session!");
        let (_, results) = context.get();


        let client : WebSession::Client = match WebSessionImpl::new() {
            Ok(session) => {
                FromServer::new(None::<LocalClient>, box session)
            }
            Err(_e) => {
                return context.fail();
            }
        };
        // we need to do this dance to upcast.
        results.set_session(UiSession::Client { client : client.client});

        context.done()
    }
}

pub struct WebSessionImpl {
    db : sqlite3::Database,
}

impl WebSessionImpl {
    pub fn new() -> sqlite3::SqliteResult<WebSessionImpl> {
        let db = try!(sqlite3::open("/var/data.db"));
        db.set_busy_timeout(1000); // try for a least a second
        Ok(WebSessionImpl {
            db : db,
        })
    }
}

impl UiSession::Server for WebSessionImpl {

}

impl WebSessionImpl {

    fn is_word(&self, word : &str) -> sqlite3::SqliteResult<bool> {

        if ! word.is_alphanumeric() { return Ok(false); }

        let cursor = try!(self.db.prepare(
            format!("SELECT * FROM Words WHERE Word = \"{}\";", word),
            &None));

        return Ok(try!(cursor.step_row()).is_some());
    }

    fn validate_def(&self, word : &str, definition : &[&str]) -> sqlite3::SqliteResult<bool> {

        if definition.len() != word.len() { return Ok(false); }

        let mut idx = 0;
        for &d in definition.iter() {
            if !(try!(self.is_word(d)) && d.len() > 0 && d[0] == word[idx]) {
                return Ok(false);
            }

            idx += 1;
        }

        return Ok(true);
    }

    fn write_def(&self, word : &str, definition : &[&str]) -> sqlite3::SqliteResult<()> {

        let time : i64 = ::time::get_time().sec;
        let mut query = StrBuf::new();
        query.push_str(format!("BEGIN; DELETE FROM Definitions WHERE Definee =\"{}\"; ", word));
        query.push_str("INSERT INTO Definitions(Definee, Idx, Definer) VALUES");
        let mut idx = 0;
        for &d in definition.iter() {
            if idx != 0 { query.push_str(","); }
            query.push_str(format!("(\"{}\", {}, \"{}\")", word, idx, d));
            idx += 1;
        }
        query.push_str(";");
        query.push_str(format!("DELETE FROM Log WHERE Word=\"{}\";", word));
        query.push_str(format!("INSERT INTO Log(Word, Timestamp) VALUES(\"{}\",{});", word, time));
        query.push_str("COMMIT;");

        println!("query: {}", query);

        try!(self.db.exec(query.as_slice()));

        return Ok(());
    }

    fn get_def(&self, word : &str) -> sqlite3::SqliteResult<~str> {

        let cursor = try!(self.db.prepare(
            format!("SELECT * FROM Definitions WHERE Definee = \"{}\";", word),
            &None));

        let mut map = HashMap::<int, ~str>::new();

        loop {
            match try!(cursor.step_row()) {
                None => break,
                Some(row) => {
                    let definer = match row.get(&"Definer".to_owned()) { &sqlite3::Text(ref t) => t.clone(), _ => fail!(), };
                    let idx = match row.get(&"Idx".to_owned()) { &sqlite3::Integer(ref i) => i.clone(), _ => fail!(), };

                    map.insert(idx, definer);
                }
            }
        }

        if map.len() != word.len() {
            return Ok("<div>this word has no definition yet</div>".to_owned());
        } else {

            let mut result = StrBuf::new();
            result.push_str("<div>");
            for idx in range::<int>(0, word.len() as int) {
                let definer : &str = map.get(&idx).as_slice();
                result.push_str(format!(" <a href=\"define?word={word}\">{word}</a> ", word=definer));
            }
            result.push_str("</div>");
            return Ok(result.into_owned());
        }
    }

    fn count_defs(&self) -> sqlite3::SqliteResult<(int, int, Vec<~str>)> {
        let cursor = try!(self.db.prepare("SELECT COUNT(*) FROM Words;", &None));
        assert!(cursor.step() == sqlite3::SQLITE_ROW);
        let num_words = cursor.get_int(0);

        let cursor = try!(self.db.prepare("SELECT COUNT(*) FROM Definitions WHERE Idx = 0;", &None));
        assert!(cursor.step() == sqlite3::SQLITE_ROW);
        let defined_words = cursor.get_int(0);

        let mut recent_words = Vec::new();
        let cursor = try!(
            self.db.prepare("SELECT Word, Timestamp FROM Log ORDER BY Timestamp DESC LIMIT 5;", &None));
        loop {
            match try!(cursor.step_row()) {
                None => break,
                Some(row) => {
                    let word : ~str = match row.get(&"Word".to_owned()) {&sqlite3::Text(ref t) => t.clone(), _ => fail!(),};
                    recent_words.push(word);
                }
            }
        }
        Ok((defined_words, num_words, recent_words))

    }


    fn construct_page_data(&self, path : &::url::Path) -> sqlite3::SqliteResult<PageData> {
        if path.path.as_slice() == "define" {

            let mut query_map = HashMap::<~str, ~str>::new();
            for &(ref k, ref v) in path.query.iter() {
                query_map.insert(k.clone(), v.clone());
            }

            let word : ~str = match query_map.find(&"word".to_owned()) {
                Some(w) if try!(self.is_word(*w)) => {
                    w.clone()
                }
                _ => {
                    return Ok(Error("that's not a word".to_owned()))
                }
            };

            match query_map.find(&"definition".to_owned()) {
                None => {
                    let def_div = try!(self.get_def(word));

                    return Ok(WordAndDef(word, def_div, None));
                }
                Some(def_query) => {

                    let definition : ~[&str] = def_query.split('+').collect();

                    if try!(self.validate_def(word, definition)) {

                        try!(self.write_def(word, definition));
                        let def_div = try!(self.get_def(word));
                        return Ok(WordAndDef(word,
                                             def_div,
                                             None));
                    } else {
                        let def_div = try!(self.get_def(word));
                        return Ok(WordAndDef(word, def_div, Some("invalid definition".to_owned())))
                    }
                }
            }

        } else {
            let (num_defined, total, recent) = try!(self.count_defs());
            return Ok(HomePage(num_defined, total, recent));
        }
    }

}

static main_css : &'static str =
    "body { font-family: Helvetica, Sans, Arial;
            font-size: medium;
             margin-left: auto;
             margin-right: auto;
             width: 600px;
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


static header : &'static str =
  r#"<head><title> acronymy </title><link rel="stylesheet" type="text/css" href="main.css" >
 <meta http-equiv="Content-Type" content="text/html;charset=utf-8" >
  </head>"#;


static lookup_form : &'static str =
      r#"<form action="define" method="get">
          <input name="word" maxlength="100"/><button>find word</button></form>"#;

fn define_form(word :&str) -> ~str {
       format!("<form action=\"define\" method=\"get\">
               <input name=\"word\" value=\"{word}\" type=\"hidden\"/>
               <input name=\"definition\" maxlength=\"2000\"/>
                   <button>submit definition</button></form>", word=word)
}

enum PageData {
    Error(~str),
    WordAndDef(~str, ~str, Option<~str>),
    HomePage(int, int, Vec<~str>),
}

fn construct_html(page_data : PageData) -> ~str {
    let mut result = StrBuf::new();
    result.push_str(format!("<html>{}<body>", header));

    static home_link : &'static str = "<a href=\"/\">home</a>";
    match page_data {
        Error(e) => {
            result.push_str(format!("<div class=\"err\"> {} </div>", e));
            result.push_str(lookup_form);
            result.push_str(home_link);
        }
        WordAndDef(word, def_div, err) => {
            result.push_str(format!("<div class=\"word\">{}</div>", word));

            result.push_str(def_div);

            match err {
                None => {}
                Some(e) => {
                    result.push_str(format!("<div class=\"err\">{}</div>", e));
                }
            }

            result.push_str(define_form(word));
            result.push_str(home_link);
        }
        HomePage(num_defined, total, recent) => {
            result.push_str("<div class=\"title\">Acronymy</div>");
            result.push_str("<div>A user-editable, acronym-only dictionary.</div>");
            result.push_str(format!("<div>So far, we have defined {} out of {} words.</div>",
                                    num_defined, total));
            if recent.len() > 0 {
                result.push_str("<div>Recently modified words: ");
                let mut idx = 0;
                for w in recent.iter() {
                    if idx != 0 {
                        result.push_str(", ");
                    }
                    result.push_str(format!("<a href=\"/define?word={word}\">{word}</a>", word=*w));
                    idx += 1;
                }
                result.push_str(".</div>");
            }
            result.push_str(lookup_form);
        }
    }

    result.push_str("</body></html>");
    result.into_owned()
}

impl WebSession::Server for WebSessionImpl {
    fn get(&mut self, mut context : WebSession::GetContext) {
        println!("GET");
        let (params, results) = context.get();
        let raw_path = params.get_path();
        let content = results.init_content();
        content.set_mime_type("text/html");

        let path = match ::url::path_from_str(raw_path) {
            Err(_e) => ::url::Path::new("".to_owned(), Vec::new(), None),
            Ok(p) => p,
        };

        println!("path = {}", raw_path);

        if raw_path == "main.css" {
            content.get_body().set_bytes(main_css.as_bytes())
        } else {
            let page_data = match self.construct_page_data(&path) {
                Err(e) => { Error(format!("database error: {} ({})", e, self.db.get_errmsg())) }
                Ok(page_data) => { page_data }
            };
            content.get_body().set_bytes(construct_html(page_data).as_bytes());
        }
        context.done()
    }
    fn post(&mut self, context : WebSession::PostContext) {
        println!("POST");
        context.done()
    }
    fn put(&mut self, context : WebSession::PutContext) {
        println!("PUT");
        context.done()
    }
    fn delete(&mut self, context : WebSession::DeleteContext) {
        println!("DELETE");
        context.done()
    }
    fn open_web_socket(&mut self, context : WebSession::OpenWebSocketContext) {
        println!("OPEN WEB SOCKET");
        context.done()
    }
}



pub struct FdStream {
    inner : Box<::std::rt::rtio::RtioFileStream:Send>,
}

impl FdStream {
    pub fn new(fd : ::libc::c_int) -> ::std::io::IoResult<FdStream> {
        ::std::rt::rtio::LocalIo::maybe_raise(|io| {
            Ok (FdStream { inner : io.fs_from_raw_fd(fd, ::std::rt::rtio::DontClose) })
        })
    }
}

impl Reader for FdStream {
    fn read(&mut self, buf : &mut [u8]) -> ::std::io::IoResult<uint> {
        self.inner.read(buf).map(|i| i as uint)
    }
}

impl Writer for FdStream {
    fn write(&mut self, buf : &[u8]) -> ::std::io::IoResult<()> {
        self.inner.write(buf)
    }
}

pub struct Restorer;

impl SturdyRefRestorer for Restorer {
    fn restore(&self, obj_id : AnyPointer::Reader) -> Option<Box<ClientHook:Send>> {
        if obj_id.is_null() {
            let client : UiView::Client = FromServer::new(None::<LocalClient>, box UiViewImpl);
            Some(client.client.hook)
        } else {
            None
        }
    }
}

pub fn main() -> ::std::io::IoResult<()> {

    let args = ::std::os::args();

    if args.len() == 4 && args[1].as_slice() == "--init" {
        println!("initializing...");
        let initdb_path = ::std::path::Path::new(args[2].as_slice());
        let proddb_path = ::std::path::Path::new(args[3].as_slice());
        println!("copying database from {} to {}", args[2], args[3]);
        try!(::std::io::fs::copy(&initdb_path, &proddb_path));
        println!("success!");
    }

    // sandstorm launches us with a connection file descriptor 3
    let ifs = try!(FdStream::new(3));
    let ofs = try!(FdStream::new(3));

    let connection_state = RpcConnectionState::new();
    connection_state.run(ifs, ofs, Restorer);

    Ok(())
}



