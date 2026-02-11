use std::path::{Path, PathBuf};

use cairo_air::utils::ProofFormat;
use cairo_air::verifier::CairoVerificationError;
use stwo::prover::ProvingError;
use stwo_cairo_adapter::vm_import::{adapt_vm_output, VmImportError};
use stwo_cairo_adapter::{log_prover_input, ProverInput};
use stwo_cairo_prover::prover::create_and_serialize_proof;
use stwo_cairo_utils::file_utils::IoErrorWithPath;
use thiserror::Error;
use tracing::{span, Level};

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
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub fn generate_proof(
    pub_json: &Path,
    priv_json: &Path,
    _verify: Option<bool>,
    _proof_format: Option<ProofFormat>,
) -> Result<PathBuf, Error> {
    let _span = span!(Level::INFO, "run").entered();

    let vm_output: ProverInput = adapt_vm_output(pub_json, priv_json)?;

    log_prover_input(&vm_output);

    let out_dir = pub_json.parent().unwrap_or_else(|| Path::new("."));
    let proof_path = out_dir.join("proof.bin");

    create_and_serialize_proof(
        vm_output,
        true,
        proof_path.clone(),
        ProofFormat::Binary,
        None
    )?;
    
    Ok(proof_path)
}