#cargo +nightly install afl

build:
	RUSTFLAGS="-Znew-llvm-pass-manager=no" cargo +nightly afl build --release

fuzz:
	cargo +nightly afl fuzz -i in -o out target/release/brilirs-fuzz