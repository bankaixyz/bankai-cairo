use alloy_primitives::FixedBytes;
use bankai_hints::types::{header::*, bls::*, *};
use beacon_types::{ExecutionPayloadHeader, MainnetEthSpec};
use bls12_381::{G1Affine, G2Affine};
use cairo_vm_base::{
    types::{felt::Felt, uint256::Uint256, uint256_32::Uint256Bits32, uint384::UInt384},
    vm::cairo_vm::Felt252,
};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

impl From<super::RecursiveEpochUpdate> for RecursiveEpochUpdateCairo {
    fn from(val: super::RecursiveEpochUpdate) -> Self {
        RecursiveEpochUpdateCairo {
            inputs: val.inputs.into(),
            outputs: val.outputs.into(),
        }
    }
}

impl From<super::RecursiveEpochOutput> for RecursiveEpochOutputsCairo {
    fn from(val: super::RecursiveEpochOutput) -> Self {
        RecursiveEpochOutputsCairo {
            beacon_header_root: Uint256(BigUint::from_bytes_be(
                val.beacon_header_root.as_slice(),
            )),
            beacon_state_root: Uint256(BigUint::from_bytes_be(
                val.beacon_state_root.as_slice(),
            )),
            beacon_height: Felt(Felt252::from(val.beacon_height)),
            n_signers: Felt(Felt252::from(val.n_signers)),
            execution_header_root: Uint256(BigUint::from_bytes_be(
                val.execution_header_root.as_slice(),
            )),
            execution_header_height: Felt(Felt252::from(val.execution_header_height)),
            current_committee_hash: Uint256(BigUint::from_bytes_be(
                val.current_committee_hash.as_slice(),
            )),
            next_committee_hash: Uint256(BigUint::from_bytes_be(
                val.next_committee_hash.as_slice(),
            )),
        }
    }
}

impl From<super::RecursiveEpochInputs> for RecursiveEpochInputsCairo {
    fn from(val: super::RecursiveEpochInputs) -> Self {

        let sync_committee_update = val.sync_committee_update.map(|s| s.into());
        let output: Option<RecursiveEpochOutputsCairo> = val.stark_proof_output.map(|s| s.into());

        RecursiveEpochInputsCairo {
            epoch_update: val.epoch_update.into(),
            sync_committee_update,
            stone_proof: val.stone_proof,
            stark_proof_output: output,
        }
    }
}

impl From<super::SyncCommitteeData> for SyncCommitteeDataCairo {
    fn from(val: super::SyncCommitteeData) -> Self {
        let branch = val
            .next_sync_committee_branch
            .iter()
            .map(|b| Uint256Bits32(BigUint::from_bytes_be(b.as_slice())))
            .collect::<Vec<Uint256Bits32>>();
        let committee_data = SyncCommitteeDataCairo {
            beacon_slot: Felt(Felt252::from(val.beacon_slot)),
            next_sync_committee_branch: branch,
            next_aggregate_sync_committee: UInt384(BigUint::from_bytes_be(
                val.next_aggregate_sync_committee.as_slice(),
            )),
            committee_keys_root: Uint256Bits32(BigUint::from_bytes_be(
                val.committee_keys_root.as_slice(),
            )),
        };

        committee_data
    }
}

impl From<super::EpochUpdate> for EpochUpdateCairo {
    fn from(val: super::EpochUpdate) -> Self {
        let beacon_header = BeaconHeaderCairo {
            slot: Uint256(BigUint::from(val.header.slot)),
            proposer_index: Uint256(BigUint::from(val.header.proposer_index)),
            parent_root: Uint256(BigUint::from_bytes_be(
                val.header.parent_root.as_slice(),
            )),
            state_root: Uint256(BigUint::from_bytes_be(
                val.header.state_root.as_slice(),
            )),
            body_root: Uint256(BigUint::from_bytes_be(
                val.header.body_root.as_slice(),
            )),
        };
        let execution_header_proof: ExecutionHeaderProofCairo = ExecutionHeaderProofCairo {
            root: Uint256(BigUint::from_bytes_be(
                val.execution_header_proof.root.as_slice(),
            )),
            path: val
                
                .execution_header_proof
                .path
                .iter()
                .map(|p| Uint256Bits32(BigUint::from_bytes_be(p.as_slice())))
                .collect::<Vec<Uint256Bits32>>(),
            leaf: Uint256(BigUint::from_bytes_be(
                val.execution_header_proof.leaf.as_slice(),
            )),
            index: Felt(Felt252::from(
                val.execution_header_proof.index,
            )),
            execution_payload_header: ExecutionPayloadHeaderCairo(
                val
                    .execution_header_proof
                    .execution_payload_header,
            )
            .to_field_roots(),
        };

        let signer_data = SignerDataCairo {
            aggregate_pub: val.aggregate_pub.into(),
            non_signers: val.non_signers.iter().map(|n| n.clone().into()).collect::<Vec<G1PointCairo>>(),
            n_non_signers: Felt(Felt252::from(val.non_signers.len() as u64)),
        };

        let inputs = EpochUpdateCairo {
            header: beacon_header,
            signature_point: val.signature_point.into(),
            signer_data,
            execution_header_proof,
        };
        // let expected_outputs = ExpectedEpochUpdateCairoOutputs {
        //     beacon_header_root: Uint256(BigUint::from_bytes_be(
        //         val.expected_circuit_outputs.beacon_header_root.as_slice(),
        //     )),
        //     beacon_state_root: Uint256(BigUint::from_bytes_be(
        //         val.expected_circuit_outputs.beacon_state_root.as_slice(),
        //     )),
        //     committee_hash: Uint256(BigUint::from_bytes_be(
        //         val.expected_circuit_outputs.committee_hash.as_slice(),
        //     )),
        //     n_signers: Felt(Felt252::from(val.expected_circuit_outputs.n_signers)),
        //     slot: Felt(Felt252::from(val.expected_circuit_outputs.slot)),
        //     execution_header_hash: Uint256(BigUint::from_bytes_be(
        //         val.expected_circuit_outputs
        //             .execution_header_hash
        //             .as_slice(),
        //     )),
        //     execution_header_height: Felt(Felt252::from(
        //         val.expected_circuit_outputs.execution_header_height,
        //     )),
        // };

        // Read and parse proof.json
        // let proof_path = Path::new("proof.json"); // Assumes proof.json is in the workspace root
        // let proof_file = File::open(proof_path).expect("Unable to open proof.json");
        // let proof_reader = BufReader::new(proof_file);
        // let proof_json: serde_json::Value = serde_json::from_reader(proof_reader).expect("Unable to parse proof.json");

        inputs
    }
}


impl From<super::G1Point> for G1PointCairo {
    fn from(val: super::G1Point) -> Self {
        let json = serde_json::to_string(&val).unwrap();
        let parsed: G1PointCairo = serde_json::from_str(&json).unwrap();
        parsed
    }
}

impl From<super::G2Point> for G2PointCairo {
    fn from(val: super::G2Point) -> Self {
        let json = serde_json::to_string(&val).unwrap();
        let parsed: G2PointCairo = serde_json::from_str(&json).unwrap();
        parsed
    }
}