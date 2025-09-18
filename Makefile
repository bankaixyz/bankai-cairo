setup:
	./scripts/setup.sh

activate:
	@echo "Please source the virtual environment activation script:"
	@echo "  source scripts/activate.sh"

build-stwo:
	./scripts/cairo_compile.sh cairo/src/bankai_stwo.cairo

build-stone:
	./scripts/cairo_compile.sh cairo/src/bankai_stone.cairo

get-program-hash:
	# @make build
	@echo "BankaiStoneProgramHash:"
	@cairo-hash-program --program cairo/build/bankai_stone.json