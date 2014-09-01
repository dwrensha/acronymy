
src/grain_capnp.rs :
	capnp compile -orust:$(OUT_DIR) --src-prefix=schema schema/grain.capnp schema/util.capnp schema/web-session.capnp
	cp acronymy_include_generated.rs $(OUT_DIR)
	rustc -L$(OUT_DIR)/../../deps $(OUT_DIR)/acronymy_include_generated.rs --out-dir $(OUT_DIR)
