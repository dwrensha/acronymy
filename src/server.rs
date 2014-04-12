use grain_capnp::{PowerboxCapability, UiView, UiSession};
use web_session_capnp::{WebSession};

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

    fn new_session(&mut self, context : UiView::NewSessionContext) {
        println!("asked for a new session!");
        context.done()
    }
}

pub struct WebSessionImpl;

impl UiSession::Server for WebSessionImpl {

}


impl WebSession::Server for WebSessionImpl {
    fn get(&mut self, context : WebSession::GetContext) {
        context.done()
    }
    fn post(&mut self, context : WebSession::PostContext) {
        context.done()
    }
    fn put(&mut self, context : WebSession::PutContext) {
        context.done()
    }
    fn delete(&mut self, context : WebSession::DeleteContext) {
        context.done()
    }
    fn open_web_socket(&mut self, context : WebSession::OpenWebSocketContext) {
        context.done()
    }
}

