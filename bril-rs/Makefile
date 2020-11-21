.PHONY: build
build:
	cargo +nightly build

.PHONY: release
release:
	cargo +nightly release

.PHONY: run
run:
	cargo +nightly run

.PHONY: format
format:
	cargo +nightly fmt

.PHONY: check
check:
	cargo +nightly check

.PHONY: example
example:
	#bril2json < ../test/interp/jmp.bril | cargo +nightly run -q | dot -Tpdf -o cfg.pdf
	bril2json < test/pointers/fib.bril | cargo +nightly run -q -- --fix_names | bril2txt
	#bril2json < benchmarks/check-primes.bril | cargo +nightly run -q -- --fix_names | dot -Tpdf -o cfg.pdf

	#| dot -Tpdf -o filename.pdf

TESTS := test/interp/*.bril \
		 test/floats/*.bril \
		 test/tdce/*.bril \
		 test/lvn/*.bril \
		 test/pointers/*.bril \
		 benchmarks/*.bril

.PHONY: test
test:
	turnt $(TURNTARGS) $(TESTS)