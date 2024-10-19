
.PHONY: all build install clean

all: build

build:
	@cargo build --release

clean:
	@rm -rf target

install:
	@cp target/release/rclean /usr/local/bin/
	@echo "rclean installed"
