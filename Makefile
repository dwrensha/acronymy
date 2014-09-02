
.PHONY : generated

generated: $(OUT_DIR)/grain_capnp.rs

$(OUT_DIR)/grain_capnp.rs : schema/grain.capnp schema/util.capnp schema/web-session.capnp
	capnp compile -orust:$(OUT_DIR) --src-prefix=schema schema/grain.capnp schema/util.capnp schema/web-session.capnp
	cp schema/acronymy_include_generated.rs $(OUT_DIR)
	rustc -L$(OUT_DIR)/../../deps $(OUT_DIR)/acronymy_include_generated.rs --out-dir $(OUT_DIR)
