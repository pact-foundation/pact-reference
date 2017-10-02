# This Makefile exists to wrap `cargo`'s `--all` command for backwards
# compatibility.

all:
	# We don't `clean` before building, because cargo is _extremely_ good at
	# tracking dependencies, and it will correctly handle even edge cases like a
	# compiler version upgrade without a manual `clean`. So it doesn't make
	# sense to recompile the entire world unless somebody specifically requests
	# it, especially given long Rust compile times.
	#
	# We call `build` and `test` separately, because they compile slightly
	# different things.
	cargo build --all
	cargo test --all

clean:
	# This doesn't require `--all`, perhaps because there's only one shared
	# artifacts directory.
	cargo clean
