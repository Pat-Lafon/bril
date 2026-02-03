.PHONY: build test clean

# Tests that brilm currently supports
# Failing tests need SSA phi node insertion (br.bril, add-overflow.bril, non_linear_control_flow.bril)
SUPPORTED_TESTS := \
	core/call-with-args.bril \
	core/call.bril \
	core/dead_block.bril \
	core/div.bril \
	core/jmp.bril \
	core/main-args.bril \
	core/main-bool-args.bril \
	core/nop.bril \
	core/ret.bril \
	core/tiny.bril

build:
	cargo build --release

test: build
	cd ../../test/interp && turnt -e brilm $(SUPPORTED_TESTS)

clean:
	cargo clean
