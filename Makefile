setup:
	./scripts/setup.sh

activate:
	@echo "Please source the virtual environment activation script:"
	@echo "  source scripts/activate.sh"

build-ethereum:
	./scripts/cairo_compile.sh cairo/src/light_clients/ethereum/main.cairo

build-bankai:
	./scripts/cairo_compile.sh cairo/src/bankai_os/main.cairo

get-program-hash:
	# @make build
	@echo "BankaiStoneProgramHash:"
	@cairo-hash-program --program cairo/build/main.json

test-block-signer:
	bash ./scripts/test_block_signer.sh

test-fixtures:
	bash ./scripts/run_fixture_tests.sh