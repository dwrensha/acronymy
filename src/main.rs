#![crate_name="acronymy"]
#![crate_type = "bin"]

extern crate libc;
extern crate time;
extern crate url;
extern crate capnp;
extern crate "capnp-rpc" as capnp_rpc;
extern crate sqlite3;

extern crate acronymy_include_generated;

pub use acronymy_include_generated::{grain_capnp, util_capnp, web_session_capnp};

pub mod server;

pub fn main() {
    match server::main() {
        Ok(()) => {}
        Err(e) => {
            println!("error: {}", e);
        }
    }
}
