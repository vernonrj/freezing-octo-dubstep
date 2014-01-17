# Rust parameters
ARCH=`uname -s`-`uname -r`-`uname -m`
SRC=.
BUILD=build
RUSTC=rustc -Z debug-info --out-dir $(BUILD) -L $(BUILD)
MAIN_FILE_SRC=$(SRC)/main.rs
# TESTS_MODULE_SRC=$(SRC)/tests.rs
MAIN_BINARY=$(BUILD)/rusp-$(ARCH)

all: build

clean:
	rm -fr $(BUILD) || true

build: clean
	mkdir $(BUILD) || true
	$(RUSTC) $(MAIN_FILE_SRC) -o $(MAIN_BINARY)

run:
	./$(MAIN_BINARY)
