#![crate_name="initdb"]
#![crate_type = "bin"]

extern crate sqlite3;

mod init {
    use sqlite3::{open, SqliteResult};

    pub fn main() -> SqliteResult<()> {
        let args = ::std::os::args();

        let mut db = try!(open(args[1].as_slice()));

        try!(db.exec("CREATE TABLE Words(Word TEXT);"));
        try!(db.exec("CREATE TABLE Definitions(Definee TEXT, Idx INTEGER, Definer TEXT);"));
        try!(db.exec("CREATE TABLE Log(Word TEXT, Timestamp INTEGER);"));

        let mut input = ::std::io::stdin();
        for line in input.lines() {
            let word = line.unwrap().clone();
            let trimmed = word.as_slice().trim();
            assert!(trimmed.is_alphanumeric(), "not alphanumeric: {}", trimmed);
            try!(db.exec(format!("INSERT INTO Words VALUES(\"{}\");", trimmed).as_slice()));
        }
        Ok(())
    }
}

pub fn main() {
    match init::main() {
        Ok(()) => {}
        Err(e) => { println!("error: {}", e) }
    }
}
