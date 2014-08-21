# Acronymy

Acronymy is a free dictionary that anyone can edit.
There is one catch: all words must be defined
as acronyms.
For example, "cat" might be defined as "cuddly adapted tiger"
and "dog" might be defined as "dutiful obedient guardian".


Acronymy is written in Rust and can be deployed as a web app on the
[Sandstorm](https://sandstorm.io) platform.


Dependencies: [capnproto-rust](https://github.com/dwrensha/capnproto-rust),
[rustsqlite](https://github.com/linuxfood/rustsqlite).


## Building

```
$ capnp compile -orust src/grain.capnp src/util.capnp src/web-session.capnp
$ rustc src/main.rs -L ~/src/capnproto-rust/ -L ~/src/rustsqlite/
```