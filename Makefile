linux:
	cross build --release --target=x86_64-unknown-linux-musl

install:
	cargo build --release
	sudo cp -f target/release/zash /usr/bin/zash