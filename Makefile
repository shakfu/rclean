
.PHONY: all build test install clean

all: build

build:
	@cargo build --release

test:
	@cargo test

clean:
	@rm -rf target

install: build
	@cp target/release/drclean /usr/local/bin/
	@echo "drclean installed"
