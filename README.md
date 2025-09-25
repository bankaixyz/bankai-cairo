# Bankai

This repository contains the Cairo implementation of Bankai, a recursive light client for Ethereum. Bankai uses STARKs to provide trustless access to Ethereum's state through a Rust-based runtime with integrated STWO prover support.

## Current Features

This Cairo implementation currently supports:

- **Beacon chain sync committee verification logic** - Validates BLS signatures from the sync committee for each consensus epoch
- **Decommit execution chain headers** - Processes and validates execution chain header data
- **Build MMR tree for beacon headers** - Constructs Merkle Mountain Range trees for beacon chain data
- **MMR for execution headers coming soon** - Execution chain MMR support is in development
- **Output generation** - All verification results and commitments are written to the output

The core logic validates the progression of the Ethereum blockchain by verifying BLS signatures from the sync committee for each consensus epoch. The program then grows the MMR (Merkle Mountain Range) tree and writes the commitments to the output. This process will eventually allow the validity of a large portion of Ethereum's history to be compressed into a single, compact STARK proof.

## Components

The repository is structured into several key components:

### Cairo Code

The Cairo source files, located in `cairo/`, implement the core logic of the light client. Key functionalities include:
-   **BLS Signature Verification**: Logic for verifying BLS signatures from the sync committee can be found in `cairo/src/bls/`.
-   **Recursive Proof Verification**: The recursion logic, which combines multiple epoch proofs, is located in `cairo/src/recursion/`. This is currently only working with the Stone prover.
-   **MMR Tree Construction**: The MMR tree construction logic is located in `cairo/packages/mmr_header_accumulator`.
-   **Main Programs**: The main entry points are `cairo/src/bankai_stone.cairo` for Stone prover and `cairo/src/bankai_stwo.cairo` for STWO prover.

### Rust Crates

The project includes several Rust crates located in the `crates/` directory to support the execution and development of the Cairo programs.

#### Cairo Runner

A dedicated Cairo runner in `crates/cairo_runner/` is used to execute the compiled Cairo programs. It loads the program and its inputs, runs the Cairo VM, and produces a Cairo PIE (Proof-Integrated Execution) file as output, which can be sent to a prover.

##### API Runner
The `cairo_runner` crate also includes a web server that exposes an API for generating PIEs. The API is implemented in `crates/cairo_runner/bin/api.rs` and provides a `/generate-pie` endpoint. This allows for remote generation of proofs without needing to run the cairo runner locally.

#### Bankai Hints

The `crates/bankai_hints/` crate provides the custom hints required by the Cairo programs to be executed by the Rust Cairo VM. These hints are used to inject inputs and handle complex computations that are not efficiently expressed in Cairo. The crate also contains Rust type definitions that mirror the Cairo structs, facilitating seamless interaction between the two languages.

#### Stone Verifier Hints

This crate, located at `crates/stone_verifier_hints/`, provides the necessary hints for the Stone verifier. It includes logic for the verifier and related data structures.

## How to Run

### Initial Setup

1. **Setup the environment:**
   ```sh
   make setup
   source scripts/activate.sh
   ```

2. **Build STWO components:**
   ```sh
   make build-stwo
   ```

### Running with STWO Prover

STWO is now working and fully integrated. To run the Cairo programs locally:

1. **Prepare your input file:**
   - Create or use an existing `input.json` file with the required input data
   - Place it in the project root directory

2. **Run with local proving:**
   ```sh
   cargo run -r --bin cairo-runner -- --input-path input.json --stwo --prove
   ```

3. **Run without local proving (generate trace only):**
   ```sh
   cargo run -r --bin cairo-runner -- --input-path input.json --stwo
   ```

The `--prove` flag enables local proving with the integrated STWO prover. Without this flag, the runner will only generate a PIE (Proof-Integrated Execution) file that can be sent to an external prover.

### API Server

The project includes a web API for remote proof generation:

1. **Start the API server:**
   ```sh
   cargo run --bin api
   ```

2. **Send requests to the `/generate-pie` endpoint** with your input data to generate PIEs remotely

### Input File Format

The `input.json` file should contain the necessary data for the light client verification, including:
- Sync committee signatures
- Beacon chain headers
- MMR tree data (for beacon chain)
- Any other required verification parameters

The exact format depends on the specific verification scenario and can be customized based on your use case.

## API Logic

The API server provides a RESTful interface for generating PIEs remotely. The main endpoint is `/generate-pie` which accepts input data and returns a PIE file that can be sent to a prover. This allows for distributed proof generation without requiring the full Cairo runtime to be installed locally.

## Docker Build Instructions

To build and push single-architecture images for the API runner:

- linux/amd64 (typical Linux servers):
```sh
docker buildx build -f Dockerfile.api --platform linux/amd64 \
  -t petscheit/bankai-runner:amd64 --push .
```

- linux/arm64 (Apple Silicon and arm64 Linux):
```sh
docker buildx build -f Dockerfile.api --platform linux/arm64 \
  -t petscheit/bankai-runner:arm64 --push .
```

For a local image without pushing (loads into your local Docker):
```sh
docker buildx build -f Dockerfile.api --platform linux/amd64 \
  -t bankai-runner:amd64 --load .
```
