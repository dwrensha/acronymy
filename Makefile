
.PHONY : clean all

clean:
	rm src *_capnp.rs

all:
	capnp compile -orust src/grain.capnp src/util.capnp src/web-session.capnp
