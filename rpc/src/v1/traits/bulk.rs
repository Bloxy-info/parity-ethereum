//! Eth rpc interface.
use jsonrpc_core::{BoxFuture};
use jsonrpc_derive::rpc;


use v1::types::{BlockNumber,  BlockWithTransactions};


/// Eth rpc interface.
#[rpc]
pub trait Bulk {
	type Metadata;

	/// Returns block with given number.
	#[rpc(name = "bulk_getBlockByNumber")]
	fn block_by_number(&self, BlockNumber) -> BoxFuture<Option<BlockWithTransactions>>;

}

