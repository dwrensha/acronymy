use std::os;
use std::io::Command;

fn main() {

    let out_dir = os::getenv("OUT_DIR").unwrap();

    let _output = Command::new("capnp")
        .arg("compile")
        .arg(format!("-orust:{}", out_dir))
        .arg("--src-prefix=schema")
        .arg("schema/grain.capnp")
        .arg("schema/util.capnp")
        .arg("schema/web-session.capnp")
        .output()
        .unwrap();

}
