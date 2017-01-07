extern crate futures;
extern crate tokio_core;
extern crate mio_uds;
extern crate time;
extern crate url;
extern crate capnp;
#[macro_use] extern crate capnp_rpc;
extern crate sandstorm;
extern crate sqlite3;

pub mod server;

fn main() {
    match server::main() {
        Ok(()) => {return;}
        Err(e) => {
            panic!("error: {}", e);
        }
    }
}
