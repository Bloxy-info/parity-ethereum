
use ethereum_types::{H160, H256, U256, Bloom as H2048};
use v1::types::{Bytes, Transaction,Receipt,LocalizedTrace};

#[derive(Debug, Serialize)]
pub struct TransactionWithReceipt {
	pub transaction: Transaction,
	pub receipt: Receipt,
	pub traces: Vec<LocalizedTrace>
}

/// Block representation
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockWithTransactions {
	/// Hash of the block
	pub hash: Option<H256>,
	/// Hash of the parent
	pub parent_hash: H256,
	/// Hash of the uncles
	#[serde(rename = "sha3Uncles")]
	pub uncles_hash: H256,
	/// Authors address
	pub author: H160,
	/// Alias of `author`
	pub miner: H160,
	/// State root hash
	pub state_root: H256,
	/// Transactions root hash
	pub transactions_root: H256,
	/// Transactions receipts root hash
	pub receipts_root: H256,
	/// Block number
	pub number: Option<U256>,
	/// Gas Used
	pub gas_used: U256,
	/// Gas Limit
	pub gas_limit: U256,
	/// Extra data
	pub extra_data: Bytes,
	/// Logs bloom
	pub logs_bloom: Option<H2048>,
	/// Timestamp
	pub timestamp: U256,
	/// Difficulty
	pub difficulty: U256,
	/// Total difficulty
	pub total_difficulty: Option<U256>,
	/// Seal fields
	pub seal_fields: Vec<Bytes>,
	/// Uncles' hashes
	pub uncles: Vec<H256>,
	/// Transactions
    pub transactions: Vec<TransactionWithReceipt>,
	/// Size in bytes
	pub size: Option<U256>,
}
