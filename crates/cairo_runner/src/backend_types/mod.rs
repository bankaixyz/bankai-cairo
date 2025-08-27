use alloy_primitives::FixedBytes;
use beacon_types::{ExecutionPayloadHeader, MainnetEthSpec};
use bls12_381::{G1Affine, G2Affine};
use serde::{Deserialize, Serialize};
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

pub mod convert;

/// Represents a single epoch update with its inputs and expected outputs
#[derive(Debug, Serialize, Deserialize)]
pub struct RecursiveEpochUpdate {
    /// Input data for the epoch circuit
    pub inputs: RecursiveEpochInputs,
    // Expected outputs after processing.
    pub outputs: RecursiveEpochOutput,
}

/// Represents the inputs for recursive epoch update processing using native types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveEpochInputs {
    /// The core epoch data
    pub epoch_update: EpochUpdate,
    /// Optional sync committee update data
    pub sync_committee_update: Option<SyncCommitteeData>,
    /// Optional stark proof from previous epoch update
    pub stone_proof: Option<serde_json::Value>,
    /// The output of the previous epoch proof. Required to decommit the output hash of the proof
    pub stark_proof_output: Option<RecursiveEpochOutput>,
}

/// Contains all necessary inputs for generating and verifying a single epoch proof (native types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochUpdate {
    /// The beacon chain block header
    pub header: BeaconHeader,
    /// BLS signature point in G2
    pub signature_point: G2Point,
    /// Aggregate public key of all validators
    #[serde(rename = "committee_pub")]
    pub aggregate_pub: G1Point,
    /// Public keys of validators who didn't sign
    pub non_signers: Vec<G1Point>,
    /// Proof of inclusion for the execution payload header
    pub execution_header_proof: ExecutionHeaderProof,
}

/// Contains sync committee update data for epoch transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCommitteeData {
    /// Beacon chain slot number
    pub beacon_slot: u64,
    /// Merkle branch for next sync committee
    pub next_sync_committee_branch: Vec<FixedBytes<32>>,
    /// Aggregated public key of next sync committee
    pub next_aggregate_sync_committee: FixedBytes<48>,
    /// Root hash of committee keys
    pub committee_keys_root: FixedBytes<32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveEpochOutput {
    pub beacon_header_root: FixedBytes<32>,
    pub beacon_state_root: FixedBytes<32>,
    pub beacon_height: u64,
    pub n_signers: u64,
    pub execution_header_root: FixedBytes<32>,
    pub execution_header_height: u64,
    pub current_committee_hash: FixedBytes<32>,
    pub next_committee_hash: FixedBytes<32>,
}

/// Represents a beacon chain block header
#[derive(Debug, Clone, Serialize, Deserialize, TreeHash)]
pub struct BeaconHeader {
    /// Slot number of the block
    pub slot: u64,
    /// Index of the block proposer
    pub proposer_index: u64,
    /// Root hash of the parent block
    pub parent_root: FixedBytes<32>,
    /// Root hash of the state
    pub state_root: FixedBytes<32>,
    /// Root hash of the block body
    pub body_root: FixedBytes<32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionHeaderProof {
    /// Root hash of the beacon block body merkle tree
    pub root: FixedBytes<32>,
    /// Merkle proof path containing the intermediate hashes
    pub path: Vec<FixedBytes<32>>,
    /// Hash of the execution payload header (leaf node)
    pub leaf: FixedBytes<32>,
    /// Position of the execution payload in the merkle tree. Should be 9.
    pub index: usize,
    /// The actual execution payload header data
    pub execution_payload_header: ExecutionPayloadHeader<MainnetEthSpec>,
    /// Slot number of the beacon block containing this payload
    pub slot: u64,
}

/// Point on the G1 curve used for public keys
#[derive(Debug, Clone)]
pub struct G1Point(pub G1Affine);

/// Point on the G2 curve used for signatures
#[derive(Debug, Clone)]
pub struct G2Point(pub G2Affine);

impl Serialize for G1Point {
    /// Serializes a G1 point to its uncompressed form
    ///
    /// Outputs x and y coordinates as hex strings
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let uncompressed = self.0.to_uncompressed();
        let mut x_bytes = [0u8; 48];
        let mut y_bytes = [0u8; 48];

        x_bytes.copy_from_slice(&uncompressed.as_ref()[0..48]);
        y_bytes.copy_from_slice(&uncompressed.as_ref()[48..96]);

        serde_json::json!({
            "x": format!("0x{}", hex::encode(x_bytes)),
            "y": format!("0x{}", hex::encode(y_bytes))
        })
        .serialize(serializer)
    }
}

impl Serialize for G2Point {
    /// Serializes a G2 point to its uncompressed form
    ///
    /// Outputs x0, x1, y0, y1 coordinates as hex strings
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let uncompressed = self.0.to_uncompressed();
        let mut x0_bytes = [0u8; 48];
        let mut x1_bytes = [0u8; 48];
        let mut y0_bytes = [0u8; 48];
        let mut y1_bytes = [0u8; 48];
        x0_bytes.copy_from_slice(&uncompressed.as_ref()[48..96]);
        x1_bytes.copy_from_slice(&uncompressed.as_ref()[0..48]);
        y0_bytes.copy_from_slice(&uncompressed.as_ref()[144..192]);
        y1_bytes.copy_from_slice(&uncompressed.as_ref()[96..144]);
        serde_json::json!({
            "x0": format!("0x{}", hex::encode(x0_bytes)),
            "x1": format!("0x{}", hex::encode(x1_bytes)),
            "y0": format!("0x{}", hex::encode(y0_bytes)),
            "y1": format!("0x{}", hex::encode(y1_bytes))
        })
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for G1Point {
    /// Deserializes a G1 point from its uncompressed form
    ///
    /// Expects x and y coordinates as hex strings
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize into a Value first
        let value: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

        // Extract x and y coordinates
        let x_str = value["x"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing x coordinate"))?;
        let y_str = value["y"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing y coordinate"))?;

        // Safely remove "0x" prefix if it exists
        let x_hex = x_str.strip_prefix("0x").unwrap_or(x_str);
        let y_hex = y_str.strip_prefix("0x").unwrap_or(y_str);

        let x_bytes = hex::decode(x_hex)
            .map_err(|e| serde::de::Error::custom(format!("invalid x hex: {e}")))?;
        let y_bytes = hex::decode(y_hex)
            .map_err(|e| serde::de::Error::custom(format!("invalid y hex: {e}")))?;

        // Combine into uncompressed format
        let mut uncompressed = [0u8; 96];
        uncompressed[0..48].copy_from_slice(&x_bytes);
        uncompressed[48..96].copy_from_slice(&y_bytes);

        // Convert to G1Affine point
        let point = G1Affine::from_uncompressed(&uncompressed).unwrap();

        Ok(G1Point(point))
    }
}

impl<'de> Deserialize<'de> for G2Point {
    /// Deserializes a G2 point from its uncompressed form
    ///
    /// Expects x0, x1, y0, y1 coordinates as hex strings
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize into a Value first
        let value: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

        // Extract coordinates
        let x0_str = value["x0"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing x0 coordinate"))?;
        let x1_str = value["x1"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing x1 coordinate"))?;
        let y0_str = value["y0"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing y0 coordinate"))?;
        let y1_str = value["y1"]
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("missing y1 coordinate"))?;

        // Safely remove "0x" prefix if it exists
        let x0_hex = x0_str.strip_prefix("0x").unwrap_or(x0_str);
        let x1_hex = x1_str.strip_prefix("0x").unwrap_or(x1_str);
        let y0_hex = y0_str.strip_prefix("0x").unwrap_or(y0_str);
        let y1_hex = y1_str.strip_prefix("0x").unwrap_or(y1_str);

        // Decode hex strings to bytes
        let x0_bytes = hex::decode(x0_hex)
            .map_err(|e| serde::de::Error::custom(format!("invalid x0 hex: {e}")))?;
        let x1_bytes = hex::decode(x1_hex)
            .map_err(|e| serde::de::Error::custom(format!("invalid x1 hex: {e}")))?;
        let y0_bytes = hex::decode(y0_hex)
            .map_err(|e| serde::de::Error::custom(format!("invalid y0 hex: {e}")))?;
        let y1_bytes = hex::decode(y1_hex)
            .map_err(|e| serde::de::Error::custom(format!("invalid y1 hex: {e}")))?;

        // Combine into uncompressed format
        let mut uncompressed = [0u8; 192];
        uncompressed[0..48].copy_from_slice(&x1_bytes);
        uncompressed[48..96].copy_from_slice(&x0_bytes);
        uncompressed[96..144].copy_from_slice(&y1_bytes);
        uncompressed[144..192].copy_from_slice(&y0_bytes);

        // Convert to G2Affine point
        let point = G2Affine::from_uncompressed(&uncompressed).unwrap();

        Ok(G2Point(point))
    }
}
