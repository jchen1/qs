.PHONY: dev clean

dev:
	RUST_BACKTRACE=1 systemfd --no-pid -s http::8080 -- cargo watch -x run
prod:
	cargo build
clean:
	rm -rf target && dropdb dev_db
