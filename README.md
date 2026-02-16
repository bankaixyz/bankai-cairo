# Bankai

This repository contains the Cairo implementation of Bankai OS, a recursive light-client OS that orchestrates modular light clients (currently Ethereum). Bankai OS composes light-client outputs into a Bankai block, verifies recursion (mock verifier for now), and is designed to compress long blockchain histories into compact STARK proofs.

## Current Features

This Cairo implementation currently supports:

- **Bankai OS block execution** - Runs light clients and outputs a new Bankai block each step
- **Ethereum light client** - Sync committee BLS verification, beacon/execution header validation
- **MMR tree construction** - Builds Merkle Mountain Range trees for beacon and execution headers
- **Standalone light client entrypoint** - Ethereum client can run independently of Bankai OS

## Architecture (brief)

- **Bankai OS core** lives in `cairo/src/bankai_os/` and defines the block format, configuration, and recursion hook
- **Light clients** live in `cairo/src/light_clients/` and each expose a `run(prev, network_id, is_genesis)` entrypoint with their own state
- **Ethereum configs** live in `cairo/src/light_clients/ethereum/config/` and select per-network fork schedule and domains
- **MMR utilities** live in `cairo/packages/mmr_header_accumulator`

## Components

The repository is structured into several key components:

### Cairo Code

The Cairo source files, located in `cairo/`, implement the core logic of Bankai OS and its light clients:
-   **Bankai OS core**: `cairo/src/bankai_os/` (main program, block format, config, recursion hook)
-   **Light clients**: `cairo/src/light_clients/` (Ethereum client and its state/types)
-   **MMR Tree Construction**: `cairo/packages/mmr_header_accumulator`
-   **Main Programs**: `cairo/src/bankai_os/main.cairo` (Bankai OS) and `cairo/src/light_clients/ethereum/main.cairo` (standalone Ethereum client)

### Rust Crates

The project includes several Rust crates located in the `crates/` directory to support the execution and development of the Cairo programs.

#### Cairo Runner

A dedicated Cairo runner in `crates/cairo_runner/` is used to execute the compiled Cairo programs. It loads the program and its inputs, runs the Cairo VM, and produces a Cairo PIE (Proof-Integrated Execution) file as output, which can be sent to a prover.

##### API Runner
The `cairo_runner` crate also includes a web server that exposes an API for generating PIEs. The API is implemented in `crates/cairo_runner/bin/api.rs` and provides a `/generate-pie` endpoint. This allows for remote generation of proofs without needing to run the cairo runner locally.

#### Bankai Hints

The `crates/bankai_hints/` crate provides the custom hints required by the Cairo programs to be executed by the Rust Cairo VM. It includes Bankai OS and light-client hints for loading inputs and mirrors Cairo structs in Rust types.

#### Stone Verifier Hints

This crate, located at `crates/stone_verifier_hints/`, provides the necessary hints for the Stone verifier. It includes logic for the verifier and related data structures.

## How to Run

### Initial Setup

1. **Setup the environment:**
   ```sh
   make setup
   source scripts/activate.sh
   ```

2. **Build Bankai components:**
   ```sh
   make build-bankai
   ```

### Running with STWO Prover

STWO is now working and fully integrated. To run the Cairo programs locally:

1. **Prepare your input file:**
   - Create or use an existing `input.json` file with the required input data
   - Place it in the project root directory
   - Compile the desired entrypoint (Bankai OS or standalone light client) before running

2. **Run with local proving:**
   ```sh
   cargo run -r --bin cairo-runner -- --input-path input.json --prove
   ```

3. **Run without local proving (generate trace only):**
   ```sh
   cargo run -r --bin cairo-runner -- --input-path input.json
   ```

The `--prove` flag enables local proving with the integrated STWO prover. Without this flag, the runner will only generate a PIE (Proof-Integrated Execution) file that can be sent to an external prover.

Entry points:
- Bankai OS: `cairo/src/bankai_os/main.cairo`
- Standalone Ethereum light client: `cairo/src/light_clients/ethereum/main.cairo`

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

## Prover Service with Docker Compose

Run the prover service with a persisted Docker volume for `prover-data`:

```sh
docker compose -f docker-compose.prover-service.yml up --build -d
```

Check logs:

```sh
docker compose -f docker-compose.prover-service.yml logs -f prover-service
```

Stop and remove container (keep volume):

```sh
docker compose -f docker-compose.prover-service.yml down
```

Stop and remove container + volume:

```sh
docker compose -f docker-compose.prover-service.yml down -v
```
