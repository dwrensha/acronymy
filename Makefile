
DEPS_DIR=$(OUT_DIR)/../../deps
CAPNP_DEP=$(shell ls $(DEPS_DIR)/libcapnp*.rlib)

.PHONY : generated

generated: $(OUT_DIR)/grain_capnp.rs

SCHEMA_SOURCES=schema/grain.capnp schema/util.capnp schema/web-session.capnp

$(OUT_DIR)/grain_capnp.rs : $(SCHEMA_SOURCES) $(CAPNP_DEP)
	capnp compile -orust:$(OUT_DIR) --src-prefix=schema $(SCHEMA_SOURCES)
	cp schema/acronymy_include_generated.rs $(OUT_DIR)
	rustc -L$(DEPS_DIR) $(OUT_DIR)/acronymy_include_generated.rs --out-dir $(OUT_DIR)
