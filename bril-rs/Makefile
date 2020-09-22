.PHONY: build
build:
	cargo +nightly build

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
	# bril2json < test/interp/alloc.bril | cargo +nightly run -q
	bril2json < test/lvn/clobber.bril | cargo +nightly run -q -- --lvn --dce | bril2txt

TESTS := test/interp/*.bril \
		 test/tdce/*.bril \
		 test/lvn/*.bril

.PHONY: test
test:
	turnt $(TURNTARGS) $(TESTS)