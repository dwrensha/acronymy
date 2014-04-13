use grain_capnp::{PowerboxCapability, UiView, UiSession};
use web_session_capnp::{WebSession};

use capnp::capability::{ClientHook, FromServer};
use capnp::AnyPointer;
use capnp_rpc::rpc::{RpcConnectionState, SturdyRefRestorer};
use capnp_rpc::capability::{LocalClient};


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

        let client : WebSession::Client = FromServer::new(None::<LocalClient>, ~WebSessionImpl);
        // we need to do this dance to upcast.
        results.set_session(UiSession::Client { client : client.client});

        context.done()
    }
}

pub struct WebSessionImpl;

impl UiSession::Server for WebSessionImpl {

}

static main_css : &'static str =
    "body { font-family: Helvetica, Sans, Arial;
            font-size: medium;
             margin-left: auto;
             margin-right: auto;
             width: 600px;
     }";


static header : &'static str =
  r#"<head><title> acronymy </title><link rel="stylesheet" type="text/css" href="main.css" >
 <meta http-equiv="Content-Type" content="text/html;charset=utf-8" >
  </head>"#;

fn html_body(body :&str) -> ~str {
    format!("<html>{}<body>{}</body></html>", header, body)
}

struct Path {
    path : ~str,
    query : Vec<(~str,Vec<~str>)>,
}

impl Path {
    fn new() -> Path {
        Path { path : ~"", query : Vec::new() }
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
            result.query.push((a[0].into_owned(),
                               a[1].split('+').map(|s| {s.into_owned() }).collect()));
        }
    }
    return result;
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
            if path.query.len() != 1 {
                return context.fail();
            }
            let &(_, ref v) = path.query.get(0);

            content.get_body().set_bytes(
                html_body(
                    "<div>AWESOME</div>
                     <form action=\"define\" method=\"get\">
                     <input name=\"word\"/><button>go</button></form>").as_bytes());

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



