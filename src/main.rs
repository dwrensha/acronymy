#![crate_id="acronymy"]
#![crate_type = "bin"]

extern crate libc;
extern crate capnp;
extern crate capnp_rpc = "capnp-rpc";

pub mod grain_capnp;
pub mod util_capnp;
pub mod web_session_capnp;

pub mod server;

pub fn main() {
    match server::main() {
        Ok(()) => {}
        Err(e) => {
            println!("error: {}", e);
        }
    }
}
