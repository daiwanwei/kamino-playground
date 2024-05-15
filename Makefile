dumpProgram:
	./deps/dump-from-mainnet.sh

localnet:
	solana-test-validator $(shell ./deps/test-validator-params.sh)

lint:
	cargo +nightly fmt