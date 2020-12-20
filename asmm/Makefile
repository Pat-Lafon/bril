.PHONY: build
build:
	cargo +nightly build

.PHONY: run
run:
	#cat benchmarks/eight-queens.bril.asmm | bril2json | cargo +nightly run -q | brili -p 8
	cat benchmarks/sieve.bril.asmm | bril2json | cargo +nightly run -q | bril2txt

.PHONY: check
check:
	cargo +nightly check

.PHONY: clippy
clippy:
	cargo +nightly clippy

TESTS := benchmarks/*.bril.asmm

.PHONY: test
test:
	turnt -v $(TURNTARGS) $(TESTS)