# Bankai

This repository contains the Cairo code for the Bankai project. Bankai is a recursive light client for Ethereum, implemented using STARKs to provide trustless access to Ethereum's state.

The core logic validates the progression of the Ethereum blockchain by verifying sync committee signatures for each consensus epoch. This process is recursive, allowing the validity of a large portion of Ethereum's history to be compressed into a single, compact STARK proof.

Currently, the Stone prover is supported for generating proofs. Support for the STWO prover is in progress.

## Components

The repository is structured into several key components:

### Cairo Code

The Cairo source files, located in `cairo/`, implement the core logic of the light client. Key functionalities include:
-   **BLS Signature Verification**: Logic for verifying BLS signatures from the sync committee can be found in `cairo/src/bls/`.
-   **Recursive Proof Verification**: The recursion logic, which combines multiple epoch proofs, is located in `cairo/src/recursion/`.
-   **Main Program**: The main entry point for the Stone prover version of the light client is `cairo/src/bankai_stone.cairo`.

### Rust Crates

The project includes several Rust crates located in the `crates/` directory to support the execution and development of the Cairo programs.

#### Cairo Runner

A dedicated Cairo runner in `crates/cairo_runner/` is used to execute the compiled Cairo programs. It loads the program and its inputs, runs the Cairo VM, and produces a Cairo PIE (Proof-Integrated Execution) file as output, which can be sent to a prover.

##### API Runner
The `cairo_runner` crate also includes a web server that exposes an API for generating PIEs. The API is implemented in `crates/cairo_runner/bin/api.rs` and provides a `/generate-pie` endpoint. This allows for remote generation of proofs without needing to run the cairo runner locally.

###### Docker Commands
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

#### Bankai Hints

The `crates/bankai_hints/` crate provides the custom hints required by the Cairo programs to be executed by the Rust Cairo VM. These hints are used to inject inputs and handle complex computations that are not efficiently expressed in Cairo. The crate also contains Rust type definitions that mirror the Cairo structs, facilitating seamless interaction between the two languages.

#### Stone Verifier Hints

This crate, located at `crates/stone_verifier_hints/`, provides the necessary hints for the Stone verifier. It includes logic for the verifier and related data structures.
