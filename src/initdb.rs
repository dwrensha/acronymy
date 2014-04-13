#![crate_id="initdb"]
#![crate_type = "bin"]

extern crate sqlite3;

mod init {
    use sqlite3::{open, SqliteResult};

    pub fn main() -> SqliteResult<()> {
        let args = ::std::os::args();

        let db = try!(open(args[1]));

        try!(db.exec("CREATE TABLE Words(Word TEXT);"));
        try!(db.exec("CREATE TABLE Definitions(Definee TEXT, Idx INTEGER, Definer TEXT);"));

        let mut input = ::std::io::stdin();
        for line in input.lines() {
            try!(db.exec(format!("INSERT INTO Words VALUES(\"{}\");", line.unwrap().trim())));
        }
        println!("hello world");
        Ok(())
    }
}

pub fn main() {
    match init::main() {
        Ok(()) => {}
        Err(e) => { println!("error: {}", e) }
    }
}
