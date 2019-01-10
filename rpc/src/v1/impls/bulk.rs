// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Bulk rpc implementation.

use std::sync::Arc;

use ethcore::client::{BlockChainClient, BlockId, TransactionId, StateClient, StateInfo, Call, EngineInfo};
use ethcore::miner::{self, MinerService};
use ethcore::encoded;

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::future;

use v1::traits::Bulk;
use v1::types::{
	BlockNumber, Bytes,
	Transaction, LocalizedTrace,
	BlockWithTransactions,TransactionWithReceipt
};
use v1::metadata::Metadata;


/// BulkClient rpc implementation.
pub struct BulkClient<C, M> where
	C: miner::BlockChainClient + BlockChainClient,
	M: MinerService {

	client: Arc<C>,
	miner: Arc<M>
}


#[derive(Debug)]
enum BlockNumberOrId {
	Number(BlockNumber),
	Id(BlockId),
}

impl From<BlockId> for BlockNumberOrId {
	fn from(value: BlockId) -> BlockNumberOrId {
		BlockNumberOrId::Id(value)
	}
}

impl From<BlockNumber> for BlockNumberOrId {
	fn from(value: BlockNumber) -> BlockNumberOrId {
		BlockNumberOrId::Number(value)
	}
}


impl<C, M, T: StateInfo + 'static> BulkClient<C, M> where
	C: miner::BlockChainClient + BlockChainClient + StateClient<State=T> + Call<State=T> + EngineInfo,
	M: MinerService<State=T> {

	/// Creates new BulkClient.
	pub fn new(
		client: &Arc<C>,
		miner: &Arc<M>
	) -> Self {
		BulkClient {
			client: client.clone(),
			miner: miner.clone()
		}
	}

	fn block(&self, id: BlockNumberOrId) -> Result<Option<BlockWithTransactions>> {
		let client = &self.client;

		let client_query = |id| (client.block(id), client.block_total_difficulty(id), client.block_extra_info(id), false);

		let (block, difficulty, _extra, is_pending) = match id {
			BlockNumberOrId::Number(BlockNumber::Pending) => {
				let info = self.client.chain_info();
				match self.miner.pending_block(info.best_block_number) {
					Some(pending_block) => {
						warn!("`Pending` is deprecated and may be removed in future versions.");

						let difficulty = {
							let latest_difficulty = self.client.block_total_difficulty(BlockId::Latest).expect("blocks in chain have details; qed");
							let pending_difficulty = self.miner.pending_block_header(info.best_block_number).map(|header| *header.difficulty());

							if let Some(difficulty) = pending_difficulty {
								difficulty + latest_difficulty
							} else {
								latest_difficulty
							}
						};

						let extra = self.client.engine().extra_info(&pending_block.header);

						(Some(encoded::Block::new(pending_block.rlp_bytes())), Some(difficulty), Some(extra), true)
					},
					None => {
						warn!("`Pending` is deprecated and may be removed in future versions. Falling back to `Latest`");
						client_query(BlockId::Latest)
					}
				}
			},

			BlockNumberOrId::Number(num) => {
				let id = match num {
					BlockNumber::Latest => BlockId::Latest,
					BlockNumber::Earliest => BlockId::Earliest,
					BlockNumber::Num(n) => BlockId::Number(n),
					BlockNumber::Pending => unreachable!() // Already covered
				};

				client_query(id)
			},

			BlockNumberOrId::Id(id) => client_query(id),
		};

		match (block, difficulty) {
			(Some(block), Some(total_difficulty)) => {
				let view = block.header_view();
				Ok(Some( BlockWithTransactions {
						hash: match is_pending {
							true => None,
							false => Some(view.hash().into()),
						},
						size: Some(block.rlp().as_raw().len().into()),
						parent_hash: view.parent_hash().into(),
						uncles_hash: view.uncles_hash().into(),
						author: view.author().into(),
						miner: view.author().into(),
						state_root: view.state_root().into(),
						transactions_root: view.transactions_root().into(),
						receipts_root: view.receipts_root().into(),
						number: match is_pending {
							true => None,
							false => Some(view.number().into()),
						},
						gas_used: view.gas_used().into(),
						gas_limit: view.gas_limit().into(),
						logs_bloom: match is_pending {
							true => None,
							false => Some(view.log_bloom().into()),
						},
						timestamp: view.timestamp().into(),
						difficulty: view.difficulty().into(),
						total_difficulty: Some(total_difficulty.into()),
						seal_fields: view.seal().into_iter().map(Into::into).collect(),
						uncles: block.uncle_hashes().into_iter().map(Into::into).collect(),
						transactions: block.view().localized_transactions().into_iter().map(|t|
							{
								let hash = t.hash().into();
								TransactionWithReceipt {
									transaction: Transaction::from_localized(t),
									receipt: self.client.transaction_receipt(TransactionId::Hash(hash)).unwrap().into(),
									traces: self.client.transaction_traces(TransactionId::Hash(hash)).map(|traces|
												traces.into_iter().map(LocalizedTrace::from).collect()
											).unwrap()
								}
							}
                        ).collect(),
						extra_data: Bytes::new(view.extra_data()),
					}))
			},
			_ => Ok(None)
		}
	}

}


impl<C, M, T: StateInfo + 'static> Bulk for BulkClient<C, M> where
	C: miner::BlockChainClient + BlockChainClient + StateClient<State=T> + Call<State=T> + EngineInfo + 'static,
	M: MinerService<State=T> + 'static,
{

	type Metadata = Metadata;

	fn block_by_number(&self, num: BlockNumber) -> BoxFuture<Option<BlockWithTransactions>> {
		Box::new(future::done(self.block(num.into())))
	}
}
