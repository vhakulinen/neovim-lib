test:
	RUST_BACKTRACE=1 cargo test

test-out:
	RUST_BACKTRACE=1 cargo test -- --nocapture

generate:
	python bindings/generate_bindings.py nvim src