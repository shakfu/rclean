
.PHONY: all build test install clean

all: build

build:
	@cargo build --release

test:
	@cargo test

clean:
	@rm -rf target

install: build
	@cp target/release/rclean /usr/local/bin/
	@echo "rclean installed"
