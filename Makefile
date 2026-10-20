.PHONY: build test check demo fmt clean

build:
	cargo build

test:
	cargo test

check:
	./scripts/check.sh

demo: build
	./scripts/demo.sh

fmt:
	cargo fmt

clean:
	rm -rf target .demo
