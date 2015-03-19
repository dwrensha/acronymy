#![crate_name="acronymy"]
#![crate_type = "bin"]

#![feature(box_syntax, collections, core, start, std_misc)]

extern crate libc;
extern crate time;
extern crate url;
extern crate capnp;
extern crate "capnp-rpc" as capnp_rpc;
extern crate sqlite3;

pub mod grain_capnp {
  include!(concat!(env!("OUT_DIR"), "/grain_capnp.rs"));
}

pub mod util_capnp {
  include!(concat!(env!("OUT_DIR"), "/util_capnp.rs"));
}

pub mod web_session_capnp {
  include!(concat!(env!("OUT_DIR"), "/web_session_capnp.rs"));
}

pub mod server;

/// We write our own entry point because Rust's default lang_start() does some
/// fancy memory protection that fails if procfs is not mounted.
#[start]
fn start(argc: isize, argv: *const *const u8) -> isize {

    // We need to do this to get ::std::env:args() to work.
    unsafe { ::std::rt::args::init(argc, argv); }

    match server::main() {
        Ok(()) => {return 0;}
        Err(e) => {
            println!("error: {}", e);
            return 1;
        }
    }
}
