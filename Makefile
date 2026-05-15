
.PHONY: all build test check clippy fmt lint doc clean install uninstall publish publish-dry

all: build

build:
	@cargo build --release

test:
	@cargo test

check:
	@cargo check

clippy:
	@cargo clippy -- -W clippy::all

fmt:
	@cargo fmt

lint: fmt clippy

doc:
	@cargo doc --no-deps --open

clean:
	@rm -rf target

install: build
	@cp target/release/rclean /usr/local/bin/
	@echo "rclean installed"

uninstall:
	@rm -f /usr/local/bin/rclean
	@echo "rclean uninstalled"

publish-dry:
	@cargo publish --dry-run

publish:
	@cargo publish
