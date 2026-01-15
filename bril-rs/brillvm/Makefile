TESTS := ../../test/interp/core/*.bril \
         ../../test/interp/float/*.bril \
		 ../../test/interp/mem/*.bril \
		 ../../test/interp/mixed/*[!r].bril # A hack to exclude store-char.bril by excluding any file ending in r.bril

BENCHMARKS := ../../benchmarks/core/*.bril \
			  ../../benchmarks/float/*.bril \
			  ../../benchmarks/mem/*.bril \
			  ../../benchmarks/mixed/*[!k].bril # a hack to exclude random_walk.bril because it uses the char extension

build:
	cargo build

clean:
	cargo clean
	cd runtime && cargo clean
	rm -f rt.bc

.PHONY: rt
rt:
	# Touch to force cargo to regenerate
	cd runtime && touch src/lib.rs && cargo rustc --lib --release -- --emit=llvm-bc
	# Move newest .bc file to rt.bc
	mv $$(ls -t runtime/target/release/deps/runtime-*.bc | head -1) rt.bc

example:
# bril2json < ../../benchmarks/mem/sieve.bril | cargo run -- -i 100
	bril2json < ../../benchmarks/mixed/cholesky.bril | cargo run -- -i

.PHONY: test
test:
	turnt -e brillvm $(TESTS)

.PHONY: benchmark
benchmark:
	turnt -e brillvm $(BENCHMARKS)