.PHONY: build
build:
	cargo +nightly build

.PHONY: release
release:
	RUSTFLAGS="-C target-cpu=native" cargo +nightly build --release

.PHONY: run
run:
	cargo +nightly run

.PHONY: clean
clean:
	cargo clean

.PHONY: format
format:
	cargo +nightly fmt

.PHONY: check
check:
	cargo +nightly check

.PHONY: clippy
clippy:
	cargo +nightly clippy

.PHONY: example
example:
	#bril2json < ../test/interp/jmp.bril | cargo +nightly run -q | dot -Tpdf -o cfg.pdf
	# bril2json < test/interp/alloc.bril | cargo +nightly run -q
	bril2json < while.bril| cargo +nightly run -- --tracing 2 | bril2txt

	#| dot -Tpdf -o filename.pdf

TESTS := test/interp/*.bril \
		 benchmarks/*.bril

.PHONY: test
test:
	turnt -v $(TURNTARGS) $(TESTS)

# todo include a thing for benchmarking