# Rust parameters
PROJECT_NAME=rusp
SRC=src
BUILD=build
RUSTC=rustc -Z debug-info -L $(BUILD)
MAIN_FILE_SRC=$(SRC)/main.rs
FILES=$(SRC)/*.rs
# TESTS_MODULE_SRC=$(SRC)/tests.rs
MAIN_BINARY=$(BUILD)/$(PROJECT_NAME)
TEST_BINARY=$(MAIN_BINARY)-test

all: $(MAIN_BINARY) $(TEST_BINARY)

clean:
	-rm -fr $(BUILD) || true

$(MAIN_BINARY): $(FILES)
	mkdir -p $(BUILD) || true
	$(RUSTC) $(MAIN_FILE_SRC) -o $(MAIN_BINARY)

$(TEST_BINARY): $(FILES)
	mkdir -p $(BUILD) || true
	$(RUSTC) $(MAIN_FILE_SRC) --test -o $(TEST_BINARY)

run: $(MAIN_BINARY)
	./$(MAIN_BINARY)

test: $(TEST_BINARY)
	./$(TEST_BINARY)
