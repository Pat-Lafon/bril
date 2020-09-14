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
	bril2json < ../test/interp/jmp.bril | cargo +nightly run -q | dot -Tpdf -o cfg.pdf
TESTS := ../test/parse/*.bril \
	../test/print/*.json \
	../test/interp*/*.bril \
	../test/ts*/*.ts \
	../test/mem/*.bril \
	../test/fail/*.bril \
	../examples/test/*/*.bril \
	../benchmarks/*.bril


.PHONY: test
test:
	turnt $(TURNTARGS) $(TESTS)