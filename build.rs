extern crate capnpc;

fn main() {
    let prefix = ::std::path::Path::new("schema");

    ::capnpc::compile(prefix,
                      &[::std::path::Path::new("schema/grain.capnp"),
                        ::std::path::Path::new("schema/util.capnp"),
                        ::std::path::Path::new("schema/web-session.capnp")]).unwrap();
}
