extern crate capnpc;

fn main() {
    let prefix = Path::new("schema");

    ::capnpc::compile(prefix.clone(),
                      vec!(Path::new("schema/grain.capnp"),
                           Path::new("schema/util.capnp"),
                           Path::new("schema/web-session.capnp")).as_slice());
}
