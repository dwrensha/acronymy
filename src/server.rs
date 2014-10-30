use grain_capnp::{powerbox_capability, ui_view, ui_session};
use web_session_capnp::{web_session};

use std::collections::hashmap::HashMap;
use capnp::capability::{ClientHook, FromServer};
use capnp::any_pointer;
use capnp_rpc::rpc::{RpcConnectionState, SturdyRefRestorer};
use capnp_rpc::capability::{LocalClient};

use sqlite3;

pub struct UiViewImpl;

impl powerbox_capability::Server for UiViewImpl {
    fn get_powerbox_info(&mut self, context : powerbox_capability::GetPowerboxInfoContext) {
        context.done()
    }
}

impl ui_view::Server for UiViewImpl {
    fn get_view_info(&mut self, context : ui_view::GetViewInfoContext) {
        context.done()
    }

    fn new_session(&mut self, mut context : ui_view::NewSessionContext) {
        println!("asked for a new session!");
        let (_, results) = context.get();


        let client : web_session::Client = match WebSessionImpl::new() {
            Ok(session) => {
                FromServer::new(None::<LocalClient>, box session)
            }
            Err(_e) => {
                return context.fail();
            }
        };
        // we need to do this dance to upcast.
        results.set_session(ui_session::Client { client : client.client});

        context.done()
    }
}

pub struct WebSessionImpl {
    db : sqlite3::Database,
}

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

        if ! word.is_alphanumeric() { return Ok(false); }

        let mut cursor = try!(self.db.prepare(
            format!("SELECT * FROM Words WHERE Word = \"{}\";", word).as_slice(),
            &None));

        return Ok(try!(cursor.step_row()).is_some());
    }

    fn validate_def(&self, word : &str, definition : &[&str]) -> sqlite3::SqliteResult<bool> {
        if definition.len() != word.len() { return Ok(false); }
        let mut idx = 0;
        for &d in definition.iter() {
            if !(try!(self.is_word(d)) && d.len() > 0 && d.char_at(0) == word.char_at(idx)) {
                return Ok(false);
            }

            idx += 1;
        }

        return Ok(true);
    }

    fn write_def(&mut self, word : &str, definition : &[&str]) -> sqlite3::SqliteResult<()> {

        let time : i64 = ::time::get_time().sec;
        let mut query = String::new();
        query.push_str(format!("BEGIN; DELETE FROM Definitions WHERE Definee =\"{}\"; ", word).as_slice());
        query.push_str("INSERT INTO Definitions(Definee, Idx, Definer) VALUES");
        let mut idx = 0u;
        for &d in definition.iter() {
            if idx != 0 { query.push_str(","); }
            query.push_str(format!("(\"{}\", {}, \"{}\")", word, idx, d).as_slice());
            idx += 1;
        }
        query.push_str(";");
        query.push_str(format!("DELETE FROM Log WHERE Word=\"{}\";", word).as_slice());
        query.push_str(format!("INSERT INTO Log(Word, Timestamp) VALUES(\"{}\",{});", word, time).as_slice());
        query.push_str("COMMIT;");

        println!("query: {}", query);

        try!(self.db.exec(query.as_slice()));

        return Ok(());
    }

    fn get_def(&self, word : &str) -> sqlite3::SqliteResult<String> {

        let mut cursor = try!(self.db.prepare(
            format!("SELECT * FROM Definitions WHERE Definee = \"{}\";", word).as_slice(),
            &None));

        let mut map = HashMap::<int, String>::new();

        loop {
            match try!(cursor.step_row()) {
                None => break,
                Some(row) => {
                    let definer = match row["Definer".to_string()] { sqlite3::Text(ref t) => t.clone(), _ => panic!(), };
                    let idx = match row["Idx".to_string()] { sqlite3::Integer(ref i) => i.clone(), _ => panic!(), };

                    map.insert(idx, definer);
                }
            }
        }

        if map.len() != word.len() {
            return Ok("<div>this word has no definition yet</div>".to_string());
        } else {

            let mut result = String::new();
            result.push_str("<div>");
            for idx in range::<int>(0, word.len() as int) {
                let definer : &str = map[idx].as_slice();
                result.push_str(format!(" <a href=\"define?word={word}\">{word}</a> ", word=definer).as_slice());
            }
            result.push_str("</div>");
            return Ok(result.into_string());
        }
    }

    fn count_defs(&self) -> sqlite3::SqliteResult<(int, int, Vec<String>)> {
        let mut cursor = try!(self.db.prepare("SELECT COUNT(*) FROM Words;", &None));
        assert!(cursor.step() == sqlite3::SQLITE_ROW);
        let num_words = cursor.get_int(0);

        let mut cursor = try!(self.db.prepare("SELECT COUNT(*) FROM Definitions WHERE Idx = 0;", &None));
        assert!(cursor.step() == sqlite3::SQLITE_ROW);
        let defined_words = cursor.get_int(0);

        let mut recent_words = Vec::new();
        let mut cursor = try!(
            self.db.prepare("SELECT Word, Timestamp FROM Log ORDER BY Timestamp DESC LIMIT 5;", &None));
        loop {
            match try!(cursor.step_row()) {
                None => break,
                Some(row) => {
                    let word : String = match row["Word".to_string()] {sqlite3::Text(ref t) => t.clone(), _ => panic!(),};
                    recent_words.push(word);
                }
            }
        }
        Ok((defined_words, num_words, recent_words))

    }


    fn construct_page_data(&mut self, path : Vec<String>, query: Option<String>) -> sqlite3::SqliteResult<PageData> {
        if path.len() == 1 && path[0].as_slice() == "define" {


            let mut query_map = HashMap::<String, String>::new();
            match query {
                None => {}
                Some(q) => {
                    for &(ref k, ref v) in ::url::form_urlencoded::parse_str(q.as_slice()).iter() {
                        query_map.insert(k.clone(), v.clone());
                    }
                }
            }

            let word : String = match query_map.find(&"word".to_string()) {
                Some(w) if try!(self.is_word(w.as_slice())) => {
                    w.clone()
                }
                _ => {
                    return Ok(Error("that's not a word".to_string()))
                }
            };
            match query_map.find(&"definition".to_string()) {
                None => {
                    let def_div = try!(self.get_def(word.as_slice()));

                    return Ok(WordAndDef(word, def_div, None));
                }
                Some(def_query) => {

                    let definition : Vec<&str> = def_query.as_slice().split(' ').collect();

                    if try!(self.validate_def(word.as_slice(), definition.as_slice())) {

                        try!(self.write_def(word.as_slice(), definition.as_slice()));
                        let def_div = try!(self.get_def(word.as_slice()));
                        return Ok(WordAndDef(word,
                                             def_div,
                                             None));
                    } else {
                        let def_div = try!(self.get_def(word.as_slice()));
                        return Ok(WordAndDef(word, def_div, Some("invalid definition".to_string())))
                    }
                }
            }

        } else {
            let (num_defined, total, recent) = try!(self.count_defs());
            return Ok(HomePage(num_defined, total, recent));
        }
    }

}

const MAIN_CSS : &'static str =
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
    HomePage(int, int, Vec<String>),
}

fn construct_html(page_data : PageData) -> String {
    let mut result = String::new();
    result.push_str(format!("<html>{}<body>", HEADER).as_slice());

    const HOME_LINK : &'static str = "<a href=\"/\">home</a>";
    match page_data {
        Error(e) => {
            result.push_str(format!("<div class=\"err\"> {} </div>", e).as_slice());
            result.push_str(LOOKUP_FORM);
            result.push_str(HOME_LINK);
        }
        WordAndDef(word, def_div, err) => {
            result.push_str(format!("<div class=\"word\">{}</div>", word).as_slice());

            result.push_str(def_div.as_slice());

            match err {
                None => {}
                Some(e) => {
                    result.push_str(format!("<div class=\"err\">{}</div>", e).as_slice());
                }
            }

            result.push_str(define_form(word.as_slice()).as_slice());
            result.push_str(HOME_LINK);
        }
        HomePage(num_defined, total, recent) => {
            result.push_str("<div class=\"title\">Acronymy</div>");
            result.push_str("<div>A user-editable, acronym-only dictionary.</div>");
            result.push_str(format!("<div>So far, we have defined {} out of {} words.</div>",
                                    num_defined, total).as_slice());
            if recent.len() > 0 {
                result.push_str("<div>Recently modified words: ");
                let mut idx = 0u;
                for w in recent.iter() {
                    if idx != 0 {
                        result.push_str(", ");
                    }
                    result.push_str(format!("<a href=\"/define?word={word}\">{word}</a>", word=*w).as_slice());
                    idx += 1;
                }
                result.push_str(".</div>");
            }
            result.push_str(LOOKUP_FORM);
        }
    }

    result.push_str("</body></html>");
    result.into_string()
}

impl web_session::Server for WebSessionImpl {
    fn get(&mut self, mut context : web_session::GetContext) {
        println!("GET");
        let (params, results) = context.get();
        let raw_path = format!("/{}", params.get_path());
        let content = results.init_content();
        content.set_mime_type("text/html");

        let (path, query) = match ::url::parse_path(raw_path.as_slice()) {
            Err(_e) => (Vec::new(), None),
            Ok((p, q, _f)) => (p, q),
        };

        println!("path = {}", raw_path);

        if raw_path.as_slice() == "/main.css" {
            content.get_body().set_bytes(MAIN_CSS.as_bytes())
        } else {
            let page_data = match self.construct_page_data(path, query) {
                Err(e) => { Error(format!("database error: {} ({})", e, self.db.get_errmsg())) }
                Ok(page_data) => { page_data }
            };
            content.get_body().set_bytes(construct_html(page_data).as_bytes());
        }
        context.done()
    }
    fn post(&mut self, context : web_session::PostContext) {
        println!("POST");
        context.done()
    }
    fn put(&mut self, context : web_session::PutContext) {
        println!("PUT");
        context.done()
    }
    fn delete(&mut self, context : web_session::DeleteContext) {
        println!("DELETE");
        context.done()
    }
    fn open_web_socket(&mut self, context : web_session::OpenWebSocketContext) {
        println!("OPEN WEB SOCKET");
        context.done()
    }
}


// copied from libstd/io/mod.rs
fn from_rtio_error(err: ::std::rt::rtio::IoError) -> ::std::io::IoError {
    let ::std::rt::rtio::IoError { code, extra, detail } = err;
    let mut ioerr = ::std::io::IoError::from_errno(code, false);
    ioerr.detail = detail;
    ioerr.kind = match ioerr.kind {
        ::std::io::TimedOut if extra > 0 => ::std::io::ShortWrite(extra),
        k => k,
    };
    return ioerr;
}


pub struct FdStream {
    inner : Box<::std::rt::rtio::RtioFileStream+Send>,
}

impl FdStream {
    pub fn new(fd : ::libc::c_int) -> ::std::io::IoResult<FdStream> {
        match ::std::rt::rtio::LocalIo::maybe_raise(|io| {
            Ok (FdStream { inner : io.fs_from_raw_fd(fd, ::std::rt::rtio::DontClose) })
        }) {
            Ok(s) => Ok(s),
            Err(e) => Err(from_rtio_error(e))
        }
    }
}

impl Reader for FdStream {
    fn read(&mut self, buf : &mut [u8]) -> ::std::io::IoResult<uint> {
        match self.inner.read(buf) {
            Ok(i) => Ok(i as uint),
            Err(e) => Err(from_rtio_error(e))
        }
    }
}

impl Writer for FdStream {
    fn write(&mut self, buf : &[u8]) -> ::std::io::IoResult<()> {
        match self.inner.write(buf) {
            Ok(()) => Ok(()),
            Err(e) => Err(from_rtio_error(e))
        }
    }
}

pub struct Restorer;

impl SturdyRefRestorer for Restorer {
    fn restore(&self, obj_id : any_pointer::Reader) -> Option<Box<ClientHook+Send>> {
        if obj_id.is_null() {
            let client : ui_view::Client = FromServer::new(None::<LocalClient>, box UiViewImpl);
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



