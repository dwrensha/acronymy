#![crate_name="initdb"]
#![crate_type = "bin"]

extern crate sqlite3;

mod init {
    use sqlite3::{open, Database, SqliteResult};

    pub fn write_db(db : &mut Database) -> SqliteResult<()> {
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

    pub fn open_db() -> SqliteResult<Database> {
        let args = ::std::os::args();
        return open(args[1].as_slice());
    }

    pub fn main() {
        match open_db() {
            Ok(mut db) => {
               match write_db(&mut db) {
                   Ok(()) => {}
                   Err(e) => { println!("error: {}, ({})", e, db.get_errmsg()) }
               }
            }
            Err(e) => {
               println!("could not open database: {}", e);
            }
        }
    }
}

pub fn main() {
    init::main();
}
