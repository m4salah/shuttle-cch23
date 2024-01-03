watch:
	@cargo-watch --watch src --watch Cargo.toml -x run
run:
	@cargo run
test:
	@cargo test -- --nocapture
watch-test:
	@cargo-watch --watch src --watch Cargo.toml -x "test -- --nocapture"