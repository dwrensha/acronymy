#[macro_use] extern crate gj;
extern crate gjio;
extern crate time;
extern crate url;
extern crate capnp;
extern crate capnp_rpc;
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
