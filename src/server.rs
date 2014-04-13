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

        let client : WebSession::Client = FromServer::new(None::<LocalClient>, ~WebSessionImpl::new());
        // we need to do this dance to upcast.
        results.set_session(UiSession::Client { client : client.client});

        context.done()
    }
}

pub struct WebSessionImpl {
    db : sqlite3::Database,
}

impl WebSessionImpl {
    pub fn new() -> WebSessionImpl {
        WebSessionImpl {
            db : sqlite3::open("/var/data.db").unwrap(),
        }
    }
}

impl UiSession::Server for WebSessionImpl {

}

static main_css : &'static str =
    "body { font-family: Helvetica, Sans, Arial;
            font-size: medium;
             margin-left: auto;
             margin-right: auto;
             width: 600px;
             text-align: center;
     }
    .word {
        text-align: center;
        font-size: 500%;
     }
     ";


static header : &'static str =
  r#"<head><title> acronymy </title><link rel="stylesheet" type="text/css" href="main.css" >
 <meta http-equiv="Content-Type" content="text/html;charset=utf-8" >
  </head>"#;

fn html_body(body :&str) -> ~str {
    format!("<html>{}<body>{}</body></html>", header, body)
}

struct Path {
    path : ~str,
    query : HashMap<~str, ~str>,
}

impl Path {
    fn new() -> Path {
        Path { path : ~"", query : HashMap::new() }
    }
}

fn parse_path(path : &str) -> Path {
    let mut result = Path::new();

    let v : ~[&str] = path.splitn('?', 2).collect();
    if v.len() == 0 {
        return result;
    }
    result.path = v[0].into_owned();
    if v.len() == 1 {
        return result;
    }
    for attr in v[1].split('&') {
        let a : ~[&str] = attr.splitn('=', 2).collect();
        if a.len() == 2 {
            result.query.insert(a[0].into_owned(),
                                a[1].into_owned());
        }
    }
    return result;
}

impl WebSessionImpl {
    fn is_word(&mut self, word : &str) -> sqlite3::SqliteResult<bool> {
        let cursor = try!(self.db.prepare(
            format!("SELECT * FROM Words WHERE Word = \"{}\";", word),
            &None));
        println!("got the cursor");
        return Ok(try!(cursor.step_row()).is_some());
    }
}

impl WebSession::Server for WebSessionImpl {
    fn get(&mut self, mut context : WebSession::GetContext) {
        println!("GET");
        let (params, results) = context.get();
        let raw_path = params.get_path();
        let content = results.init_content();
        content.set_mime_type("text/html");

        let path = parse_path(raw_path);
        println!("path = {}", raw_path);
        println!("{}, {}", path.path, path.query);

        if raw_path == "main.css" {
            content.get_body().set_bytes(main_css.as_bytes())
        } else if path.path.as_slice() == "define" {
            let word : ~str = path.query.get(&~"word").clone();

            // TODO check that `word` is actually a word.
            match self.is_word(word) {
                Err(e) => fail!("is_word error: {}", e),
                Ok(false) => {
                    content.get_body().set_bytes(
                        html_body(
                            "<div> that's not a word </div>
                           <form action=\"define\" method=\"get\">
                           <input name=\"word\"/><button>go</button></form>").as_bytes());
                    return context.done();
                }
                Ok(true) => {}
            }

            content.get_body().set_bytes(
                html_body(
                    format!(
                        "<div class=\"word\">{word}</div>
                     <form action=\"define\" method=\"get\">
                     <input name=\"word\" value=\"{word}\" type=\"hidden\"/>
                     <input name=\"definition\"/><button>define</button></form>",
                        word=word)).as_bytes());

        } else {
            content.get_body().set_bytes(
                html_body(
                    "<form action=\"define\" method=\"get\">
                     <input name=\"word\"/><button>go</button></form>").as_bytes());
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
    inner : ~::std::rt::rtio::RtioFileStream:Send,
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
    fn restore(&self, obj_id : AnyPointer::Reader) -> Option<~ClientHook:Send> {
        if obj_id.is_null() {
            let client : UiView::Client = FromServer::new(None::<LocalClient>, ~UiViewImpl);
            Some(client.client.hook)
        } else {
            None
        }
    }
}

pub fn main() -> ::std::io::IoResult<()> {
    let ifs = try!(FdStream::new(3));
    let ofs = try!(FdStream::new(3));

    let connection_state = RpcConnectionState::new();
    connection_state.run(ifs, ofs, Restorer);

    Ok(())
}



