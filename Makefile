setup:
	./scripts/setup.sh

activate:
	@echo "Please source the virtual environment activation script:"
	@echo "  source scripts/activate.sh"

build-stwo:
	./scripts/cairo_compile.sh cairo/src/bankai_stwo.cairo

build-ethereum:
	./scripts/cairo_compile.sh cairo/bankai_new/light_clients/ethereum/main.cairo

build-bankai-os:
	./scripts/cairo_compile.sh cairo/bankai_new/bankai_os/main.cairo

build-stone:
	./scripts/cairo_compile.sh cairo/src/bankai_stone.cairo

get-program-hash:
	# @make build
	@echo "BankaiStoneProgramHash:"
	@cairo-hash-program --program cairo/build/main.json

test-block-signer:
	bash ./scripts/test_block_signer.sh

test-fixtures:
	bash ./scripts/run_fixture_tests.sh