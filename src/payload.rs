use alloy_primitives::{Address, B256, Bloom, Bytes, U256, map::foldhash::HashMap};
use alloy_rpc_types_engine::PayloadId;
use alloy_rpc_types_eth::Withdrawal;
use reth_node_api::NodePrimitives;
use reth_optimism_primitives::OpPrimitives;
use serde::{Deserialize, Serialize};

/// Represents the modified portions of an execution payload within a flashblock.
/// This structure contains only the fields that can be updated during block construction,
/// such as state root, receipts, logs, and new transactions. Other immutable block fields
/// like parent hash and block number are excluded since they remain constant throughout
/// the block's construction.
#[derive(Clone, Debug, Default, Deserialize, Serialize, ssz_derive::Encode, ssz_derive::Decode)]
pub struct ExecutionPayloadFlashblockDeltaV1 {
    /// The state root of the block.
    pub state_root: B256,
    /// The receipts root of the block.
    pub receipts_root: B256,
    /// The logs bloom of the block.
    pub logs_bloom: Bloom,
    /// The gas used of the block.
    #[serde(with = "alloy_serde::quantity")]
    pub gas_used: u64,
    /// The block hash of the block.
    pub block_hash: B256,
    /// The transactions of the block.
    pub transactions: Vec<Bytes>,
    /// Array of [`Withdrawal`] enabled with V2
    pub withdrawals: Vec<Withdrawal>,
    /// The withdrawals root of the block.
    pub withdrawals_root: B256,
}

/// Represents the base configuration of an execution payload that remains constant
/// throughout block construction. This includes fundamental block properties like
/// parent hash, block number, and other header fields that are determined at
/// block creation and cannot be modified.
#[derive(Clone, Debug, Default, Deserialize, Serialize, ssz_derive::Encode, ssz_derive::Decode)]
pub struct ExecutionPayloadBaseV1 {
    /// Ecotone parent beacon block root
    pub parent_beacon_block_root: B256,
    /// The parent hash of the block.
    pub parent_hash: B256,
    /// The fee recipient of the block.
    pub fee_recipient: Address,
    /// The previous randao of the block.
    pub prev_randao: B256,
    /// The block number.
    #[serde(with = "alloy_serde::quantity")]
    pub block_number: u64,
    /// The gas limit of the block.
    #[serde(with = "alloy_serde::quantity")]
    pub gas_limit: u64,
    /// The timestamp of the block.
    #[serde(with = "alloy_serde::quantity")]
    pub timestamp: u64,
    /// The extra data of the block.
    pub extra_data: Bytes,
    /// The base fee per gas of the block.
    pub base_fee_per_gas: U256,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, ssz_derive::Encode, ssz_derive::Decode)]
pub struct FlashblocksPayloadV1 {
    /// The payload id of the flashblock
    #[ssz(with = "payload_id_ssz")]
    pub payload_id: PayloadId,
    /// The index of the flashblock in the block
    pub index: u64,
    /// The base execution payload configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<ExecutionPayloadBaseV1>,
    /// The delta/diff containing modified portions of the execution payload
    pub diff: ExecutionPayloadFlashblockDeltaV1,
    /// Additional metadata associated with the flashblock
    pub metadata: FlashblocksMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ssz_derive::Encode, ssz_derive::Decode)]
pub struct FlashblocksMetadata {
    #[ssz(with = "receipts_ssz")]
    receipts: HashMap<B256, <OpPrimitives as NodePrimitives>::Receipt>,

    #[ssz(with = "new_account_balances_ssz")]
    new_account_balances: HashMap<Address, U256>,

    block_number: u64,
}

pub mod new_account_balances_ssz {
    pub mod encode {
        use alloy_primitives::{Address, U256, map::foldhash::HashMap};
        use ssz::{BYTES_PER_LENGTH_OFFSET, Encode};

        pub fn is_ssz_fixed_len() -> bool {
            false
        }

        pub fn ssz_fixed_len() -> usize {
            BYTES_PER_LENGTH_OFFSET
        }

        pub fn ssz_bytes_len(new_account_balances: &HashMap<Address, U256>) -> usize {
            as_ssz_bytes(new_account_balances).len()
        }

        pub fn ssz_append(new_account_balances: &HashMap<Address, U256>, buf: &mut Vec<u8>) {
            for (address, balance) in new_account_balances {
                buf.extend_from_slice(address.as_slice());
                buf.extend_from_slice(balance.as_ssz_bytes().as_slice());
            }
        }

        pub fn as_ssz_bytes(new_account_balances: &HashMap<Address, U256>) -> Vec<u8> {
            let mut buf = vec![];
            ssz_append(new_account_balances, &mut buf);
            buf
        }
    }

    pub mod decode {
        use alloy_primitives::{
            Address, U256,
            map::foldhash::{HashMap, HashMapExt},
        };
        use ssz::{BYTES_PER_LENGTH_OFFSET, Decode, DecodeError};

        pub fn is_ssz_fixed_len() -> bool {
            false
        }

        pub fn ssz_fixed_len() -> usize {
            BYTES_PER_LENGTH_OFFSET
        }

        pub fn from_ssz_bytes(bytes: &[u8]) -> Result<HashMap<Address, U256>, DecodeError> {
            let mut new_account_balances = HashMap::new();
            let mut offset = 0;
            while offset < bytes.len() {
                let address = Address::from_slice(&bytes[offset..offset + 20]);
                offset += 20;
                let balance = U256::from_ssz_bytes(&bytes[offset..offset + 32])?;
                offset += 32;
                new_account_balances.insert(address, balance);
            }
            Ok(new_account_balances)
        }
    }
}

pub mod receipts_ssz {
    pub mod encode {
        use alloy_primitives::{B256, map::foldhash::HashMap};
        use reth_node_api::NodePrimitives;
        use reth_optimism_primitives::OpPrimitives;
        use ssz::BYTES_PER_LENGTH_OFFSET;

        pub fn is_ssz_fixed_len() -> bool {
            false
        }

        pub fn ssz_fixed_len() -> usize {
            BYTES_PER_LENGTH_OFFSET
        }

        pub fn ssz_bytes_len(
            receipts: &HashMap<B256, <OpPrimitives as NodePrimitives>::Receipt>,
        ) -> usize {
            as_ssz_bytes(receipts).len()
        }

        pub fn ssz_append(
            receipts: &HashMap<B256, <OpPrimitives as NodePrimitives>::Receipt>,
            buf: &mut Vec<u8>,
        ) {
            for (receipt_hash, receipt) in receipts {
                buf.extend_from_slice(receipt_hash.as_slice());
                let receipt_json_bytes = serde_json::to_vec(receipt).unwrap();
                let receipt_json_bytes_len = receipt_json_bytes.len();
                buf.extend_from_slice(&receipt_json_bytes_len.to_be_bytes());
                buf.extend_from_slice(&receipt_json_bytes);
            }
        }

        pub fn as_ssz_bytes(
            receipts: &HashMap<B256, <OpPrimitives as NodePrimitives>::Receipt>,
        ) -> Vec<u8> {
            let mut buf = vec![];
            ssz_append(receipts, &mut buf);
            buf
        }
    }

    pub mod decode {
        use alloy_primitives::{
            B256,
            map::foldhash::{HashMap, HashMapExt},
        };
        use reth_node_api::NodePrimitives;
        use reth_optimism_primitives::OpPrimitives;
        use ssz::{BYTES_PER_LENGTH_OFFSET, DecodeError};

        pub fn is_ssz_fixed_len() -> bool {
            false
        }

        pub fn ssz_fixed_len() -> usize {
            BYTES_PER_LENGTH_OFFSET
        }

        pub fn from_ssz_bytes(
            bytes: &[u8],
        ) -> Result<HashMap<B256, <OpPrimitives as NodePrimitives>::Receipt>, DecodeError> {
            let mut receipts = HashMap::new();
            let mut offset = 0;
            while offset < bytes.len() {
                let receipt_hash = B256::from_slice(&bytes[offset..offset + 32]);
                offset += 32;
                let receipt_json_bytes_len =
                    usize::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
                offset += 4;
                let receipt_json_bytes = &bytes[offset..offset + receipt_json_bytes_len];
                offset += receipt_json_bytes_len;
                let receipt: <OpPrimitives as NodePrimitives>::Receipt =
                    serde_json::from_slice(receipt_json_bytes).unwrap();
                receipts.insert(receipt_hash, receipt);
            }

            Ok(receipts)
        }
    }
}

pub mod payload_id_ssz {
    pub mod encode {
        use alloy_rpc_types_engine::PayloadId;

        pub fn is_ssz_fixed_len() -> bool {
            true
        }

        pub fn ssz_fixed_len() -> usize {
            8
        }

        pub fn ssz_bytes_len(payload_id: &PayloadId) -> usize {
            as_ssz_bytes(payload_id).len()
        }

        pub fn ssz_append(payload_id: &PayloadId, buf: &mut Vec<u8>) {
            buf.extend_from_slice(&payload_id.0.0);
        }

        pub fn as_ssz_bytes(payload_id: &PayloadId) -> Vec<u8> {
            let mut buf = vec![];
            ssz_append(payload_id, &mut buf);
            buf
        }
    }

    pub mod decode {
        use alloy_primitives::B64;
        use alloy_rpc_types_engine::PayloadId;
        use ssz::DecodeError;

        pub fn is_ssz_fixed_len() -> bool {
            true
        }

        pub fn ssz_fixed_len() -> usize {
            8
        }

        pub fn from_ssz_bytes(bytes: &[u8]) -> Result<PayloadId, DecodeError> {
            let b64_value = B64::from_slice(bytes);
            Ok(PayloadId(b64_value.into()))
        }
    }
}
