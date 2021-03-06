# Acronymy

Acronymy is a free dictionary that anyone can edit.
There is one catch: all words must be defined
as acronyms.
For example, "cat" might be defined as "cuddly adapted tiger"
and "dog" might be defined as "dutiful obedient guardian".

[![Try it on Sandstorm](https://img.shields.io/badge/try-live%20demo-783189.svg)](https://demo.sandstorm.io/appdemo/7m2fcfn7qdyexs3jmn6vrdngcryeuc8y4fa6jpyj4fgwh5tq27ph) 

Acronymy is written in Rust and can be deployed as a web app on the
[Sandstorm](https://sandstorm.io) platform.


## Building

Make sure that you have [Cap'n Proto](https://github.com/kentonv/capnproto) installed.

You must supply your own word list.

```
$ cargo build --release
$ ./target/release/initdb data.db < words.txt
$ spk dev
```
