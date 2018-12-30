.PHONY: dev clean

dev:
	RUST_BACKTRACE=1 systemfd --no-pid -s http::8081 -- cargo watch -x run& PORT=8080 yarn start
prod:
	cargo build
clean:
	rm -rf target && dropdb dev_db
