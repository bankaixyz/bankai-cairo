use crate::error::Error;
use crate::stone::transform::TransformTo;
use starknet_core::types::Felt;
use swiftness_air::layout::dynamic::Layout;
pub use swiftness_proof_parser::*;
pub use swiftness_stark::{self, types::StarkProof};

pub fn verify_proof(proof: &serde_json::Value) -> Result<Vec<Felt>, Error> {
    let stark_proof: StarkProof = parse(proof.to_string().as_str())
        .map_err(|e| Error::ProofParsingError(e.to_string()))?
        .transform_to();

    let security_bits = stark_proof.config.security_bits();
    let (bootloader_hash, program_output) = stark_proof
        .verify::<Layout>(security_bits)
        .map_err(|e| Error::ProofVerificationError(e.to_string()))?;

    // Ensure we used the correct bootloader hash
    assert_eq!(
        bootloader_hash,
        Felt::from_hex_unchecked(
            "0x5bfa89872e1bf7630bedb115afcaa176e9570b13e055a416a90e4f53b6e1468"
        )
    );

    Ok(program_output)
}
