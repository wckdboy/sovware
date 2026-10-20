.PHONY: build test check demo eval fmt clean

build:
	cargo build

test:
	cargo test

check:
	./scripts/check.sh

demo: build
	./scripts/demo.sh

eval: build
	./target/debug/sov eval examples/bracket/eval.json

fmt:
	cargo fmt

clean:
	rm -rf target .demo
