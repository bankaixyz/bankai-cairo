use std::io::Write;
use std::path::{Path, PathBuf};

use cairo_air::verifier::{verify_cairo, CairoVerificationError};
use cairo_air::PreProcessedTraceVariant;
use serde::Serialize;
use stwo::core::channel::MerkleChannel;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleChannel;
use stwo::core::vcs::poseidon252_merkle::Poseidon252MerkleChannel;
use stwo::core::vcs::MerkleHasher;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::backend::BackendForChannel;
use stwo::prover::ProvingError;
use stwo_cairo_adapter::vm_import::{adapt_vm_output, VmImportError};
use stwo_cairo_adapter::{log_prover_input, ProverInput};
use stwo_cairo_prover::prover::{prove_cairo, ChannelHash, ProverParameters};
use stwo_cairo_serialize::CairoSerialize;
use stwo_cairo_utils::file_utils::{create_file, IoErrorWithPath};
use thiserror::Error;
use tracing::{span, Level};

#[derive(Debug, Clone, Copy)]
pub enum ProofFormat {
    /// Standard JSON format.
    Json,
    /// Array of field elements serialized as hex strings.
    /// Compatible with `scarb execute`
    CairoSerde,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO failed: {0}")]
    IO(#[from] std::io::Error),
    #[error("Proving failed: {0}")]
    Proving(#[from] ProvingError),
    #[error("Serialization failed: {0}")]
    Serializing(#[from] sonic_rs::error::Error),
    #[error("Verification failed: {0}")]
    Verification(#[from] CairoVerificationError),
    #[error("VM import failed: {0}")]
    VmImport(#[from] VmImportError),
    #[error("File IO failed: {0}")]
    File(#[from] IoErrorWithPath),
}

pub fn generate_proof(
    pub_json: &Path,
    priv_json: &Path,
    verify: Option<bool>,
    proof_format: Option<ProofFormat>,
) -> Result<PathBuf, Error> {
    let _span = span!(Level::INFO, "run").entered();

    let vm_output: ProverInput = adapt_vm_output(pub_json, priv_json)?;

    log_prover_input(&vm_output);

    // Hardcode prover parameters
    let proof_params = ProverParameters {
        channel_hash: ChannelHash::Blake2s,
        pcs_config: PcsConfig {
            // Stay within 500ms on M3.
            pow_bits: 26,
            fri_config: FriConfig {
                log_last_layer_degree_bound: 0,
                // Blowup factor > 1 significantly degrades proving speed.
                // Can be in range [1, 16].
                log_blowup_factor: 1,
                // The more FRI queries, the larger the proof.
                // Proving time is not affected much by increasing this value.
                n_queries: 70,
            },
        },
        preprocessed_trace: PreProcessedTraceVariant::CanonicalWithoutPedersen,
    };

    let run_inner_fn = match proof_params.channel_hash {
        ChannelHash::Blake2s => run_inner::<Blake2sMerkleChannel>,
        ChannelHash::Poseidon252 => run_inner::<Poseidon252MerkleChannel>,
    };

    let out_dir = pub_json.parent().unwrap_or_else(|| Path::new("."));
    let proof_path = out_dir.join("proof.json");

    run_inner_fn(
        vm_output,
        proof_params.pcs_config,
        proof_params.preprocessed_trace,
        verify.unwrap_or(false),
        proof_path.clone(),
        proof_format.unwrap_or(ProofFormat::Json),
    )?;

    Ok(proof_path)
}

/// Generates proof given the Cairo VM output and prover config/parameters.
/// Serializes the proof as JSON and write to the output path.
/// Verifies the proof in case the respective flag is set.
fn run_inner<MC: MerkleChannel>(
    vm_output: ProverInput,
    pcs_config: PcsConfig,
    preprocessed_trace: PreProcessedTraceVariant,
    verify: bool,
    proof_path: PathBuf,
    proof_format: ProofFormat,
) -> Result<(), Error>
where
    SimdBackend: BackendForChannel<MC>,
    MC::H: Serialize,
    <MC::H as MerkleHasher>::Hash: CairoSerialize,
{
    let proof = prove_cairo::<MC>(vm_output, pcs_config, preprocessed_trace)?;
    let mut proof_file = create_file(&proof_path)?;

    let span = span!(Level::INFO, "Serialize proof").entered();
    match proof_format {
        ProofFormat::Json => {
            proof_file.write_all(sonic_rs::to_string_pretty(&proof)?.as_bytes())?;
        }
        ProofFormat::CairoSerde => {
            let mut serialized: Vec<starknet_ff::FieldElement> = Vec::new();
            CairoSerialize::serialize(&proof, &mut serialized);

            let hex_strings: Vec<String> = serialized
                .into_iter()
                .map(|felt| format!("0x{felt:x}"))
                .collect();

            proof_file.write_all(sonic_rs::to_string_pretty(&hex_strings)?.as_bytes())?;
        }
    }
    span.exit();
    if verify {
        verify_cairo::<MC>(proof, preprocessed_trace)?;
        tracing::info!("Proof verified successfully");
    }

    Ok(())
}
