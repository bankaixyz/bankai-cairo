use cairo_runner::{
    recursive_epoch::{
        BeaconHeaderCairo, EpochUpdateCairo, ExecutionHeaderProofCairo,
        ExecutionPayloadHeaderCairo, RecursiveEpochInputsCairo, RecursiveEpochOutputsCairo,
        RecursiveEpochUpdateCairo, SyncCommitteeDataCairo,
    },
    types::{Felt, G1PointCairo, G2PointCairo, UInt384, Uint256, Uint256Bits32},
};
use cairo_vm::Felt252;
use num_bigint::BigUint;

use crate::fetcher::recursive_epoch_input::{
    EpochUpdate, G1Point, G2Point, RecursiveEpochInputs, RecursiveEpochOutput, RecursiveEpochUpdate,
};
use crate::fetcher::sync_committee_input::SyncCommitteeData;

impl From<RecursiveEpochUpdate> for RecursiveEpochUpdateCairo {
    fn from(val: RecursiveEpochUpdate) -> Self {
        RecursiveEpochUpdateCairo {
            inputs: val.inputs.into(),
            outputs: val.outputs.into(),
        }
    }
}

impl From<RecursiveEpochOutput> for RecursiveEpochOutputsCairo {
    fn from(val: RecursiveEpochOutput) -> Self {
        RecursiveEpochOutputsCairo {
            beacon_header_root: Uint256(BigUint::from_bytes_be(val.beacon_header_root.as_slice())),
            beacon_state_root: Uint256(BigUint::from_bytes_be(val.beacon_state_root.as_slice())),
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

impl From<RecursiveEpochInputs> for RecursiveEpochInputsCairo {
    fn from(val: RecursiveEpochInputs) -> Self {
        let sync_committee_update = val.sync_committee_update.map(|s| s.into());
        let output: Option<RecursiveEpochOutputsCairo> = val.stark_proof_output.map(|s| s.into());

        RecursiveEpochInputsCairo {
            epoch_update: val.epoch_update.into(),
            sync_committee_update,
            stark_proof: val.stark_proof,
            stark_proof_output: output,
        }
    }
}

impl From<SyncCommitteeData> for SyncCommitteeDataCairo {
    fn from(val: SyncCommitteeData) -> Self {
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

impl From<EpochUpdate> for EpochUpdateCairo {
    fn from(val: EpochUpdate) -> Self {
        let beacon_header = BeaconHeaderCairo {
            slot: Uint256(BigUint::from(val.header.slot)),
            proposer_index: Uint256(BigUint::from(val.header.proposer_index)),
            parent_root: Uint256(BigUint::from_bytes_be(val.header.parent_root.as_slice())),
            state_root: Uint256(BigUint::from_bytes_be(val.header.state_root.as_slice())),
            body_root: Uint256(BigUint::from_bytes_be(val.header.body_root.as_slice())),
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
            index: Felt(Felt252::from(val.execution_header_proof.index)),
            execution_payload_header: ExecutionPayloadHeaderCairo(
                val.execution_header_proof.execution_payload_header,
            )
            .to_field_roots(),
        };
        let inputs = EpochUpdateCairo {
            header: beacon_header,
            signature_point: val.signature_point.into(),
            aggregate_pub: val.aggregate_pub.into(),
            non_signers: val
                .non_signers
                .iter()
                .map(|n| n.clone().into())
                .collect::<Vec<G1PointCairo>>(),
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

impl From<G1Point> for G1PointCairo {
    fn from(val: G1Point) -> Self {
        let json = serde_json::to_string(&val).unwrap();
        let parsed: G1PointCairo = serde_json::from_str(&json).unwrap();
        parsed
    }
}

impl From<G2Point> for G2PointCairo {
    fn from(val: G2Point) -> Self {
        let json = serde_json::to_string(&val).unwrap();
        let parsed: G2PointCairo = serde_json::from_str(&json).unwrap();
        parsed
    }
}
