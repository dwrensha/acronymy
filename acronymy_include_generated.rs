/*
 * This is a hack to allow us to do capnpc-rust code generation and still use Cargo.
 */


#![crate_name="acronymy_include_generated"]
#![crate_type = "lib"]

extern crate capnp;

pub mod grain_capnp;
pub mod util_capnp;
pub mod web_session_capnp;

