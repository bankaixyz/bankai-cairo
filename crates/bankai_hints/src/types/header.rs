use beacon_types::TreeHash;
use beacon_types::{ExecutionPayloadHeader, MainnetEthSpec};
use cairo_vm_base::types::{felt::Felt, uint256::Uint256, uint384::UInt384};


pub struct ExecutionPayloadHeaderCairo(pub ExecutionPayloadHeader<MainnetEthSpec>);

impl ExecutionPayloadHeaderCairo {
    pub fn to_field_roots(&self) -> Vec<Uint256> {
        // Helper function to convert any value to a padded 32-byte Uint256
        fn to_uint256<T: AsRef<[u8]>>(bytes: T) -> Uint256 {
            let mut padded = vec![0; 32];
            let bytes = bytes.as_ref();
            // Copy bytes to the end of the padded array (left padding with zeros)
            padded[32 - bytes.len()..].copy_from_slice(bytes);
            let value = Uint256::from_bytes_be(&padded);
            value
        }

        pub fn u64_to_uint256(value: u64) -> Uint256 {
            let mut bytes = [0u8; 32];
            // Place u64 value in the first 8 bytes (little-endian)
            bytes[0..8].copy_from_slice(&value.to_le_bytes());
            Uint256::from_bytes_be(&bytes)
        }

        macro_rules! extract_common_fields {
            ($h:expr) => {
                vec![
                    to_uint256($h.parent_hash.0.as_slice()),
                    to_uint256($h.fee_recipient.0.to_vec()),
                    to_uint256($h.state_root.0.to_vec()),
                    to_uint256($h.receipts_root.0.to_vec()),
                    to_uint256($h.logs_bloom.tree_hash_root().as_slice()),
                    to_uint256($h.prev_randao.0.to_vec()),
                    u64_to_uint256($h.block_number),
                    u64_to_uint256($h.gas_limit),
                    u64_to_uint256($h.gas_used),
                    u64_to_uint256($h.timestamp),
                    to_uint256($h.extra_data.tree_hash_root().as_slice()),
                    to_uint256($h.base_fee_per_gas.tree_hash_root().as_slice()),
                    to_uint256($h.block_hash.0.as_slice()),
                    to_uint256($h.transactions_root.as_slice()),
                ]
            };
        }

        let roots = match &self.0 {
            ExecutionPayloadHeader::Bellatrix(h) => extract_common_fields!(h),
            ExecutionPayloadHeader::Capella(h) => {
                let mut roots = extract_common_fields!(h);
                roots.push(to_uint256(h.withdrawals_root.as_slice()));
                roots
            }
            ExecutionPayloadHeader::Deneb(h) => {
                let mut roots = extract_common_fields!(h);
                roots.push(to_uint256(h.withdrawals_root.as_slice()));
                roots.push(u64_to_uint256(h.blob_gas_used));
                roots.push(u64_to_uint256(h.excess_blob_gas));
                roots
            }
            ExecutionPayloadHeader::Electra(h) => {
                // The execution payload is the same as Deneb
                let mut roots = extract_common_fields!(h);
                roots.push(to_uint256(h.withdrawals_root.as_slice()));
                roots.push(u64_to_uint256(h.blob_gas_used));
                roots.push(u64_to_uint256(h.excess_blob_gas));
                roots
            }
            ExecutionPayloadHeader::Fulu(_h) => panic!("Fulu not supported"),
        };

        roots
    }
}