// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use bonsai_proxy_contract::{Callback, ProxyContract};
use bonsai_sdk::alpha::Client;
use ethers::prelude::*;
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::{sync::Notify, task::JoinHandle};
use tracing::info;

use crate::{
    storage::{ProofRequestState, Storage},
    uploader::completed_proofs::{
        complete_proof::{get_complete_proof, CompleteProof},
        error::*,
    },
};

const BONSAI_RELAY_GAS_LIMIT: u64 = 3000000;

pub(crate) struct BonsaiCompleteProofManager<S: Storage, M: Middleware> {
    client: Client,
    storage: S,
    new_complete_proofs_notifier: Arc<Notify>,
    ready_to_send_batch: Vec<CompleteProof>,
    max_batch_size: usize,
    proxy_contract_address: Address,
    ethers_client: Arc<M>,
    send_batch_notifier: Arc<Notify>,
    send_batch_interval: tokio::time::Interval,
    futures_set: FuturesUnordered<JoinHandle<Result<CompleteProof, CompleteProofError>>>,
}

impl<S: Storage, M: Middleware + 'static> BonsaiCompleteProofManager<S, M> {
    pub(crate) fn new(
        client: Client,
        storage: S,
        new_complete_proofs_notifier: Arc<Notify>,
        send_batch_notifier: Arc<Notify>,
        max_batch_size: usize,
        proxy_contract_address: Address,
        ethers_client: Arc<M>,
        send_batch_interval: tokio::time::Interval,
    ) -> Self {
        Self {
            client,
            storage,
            new_complete_proofs_notifier,
            ready_to_send_batch: Vec::new(),
            max_batch_size,
            proxy_contract_address,
            ethers_client,
            send_batch_notifier,
            send_batch_interval,
            futures_set: FuturesUnordered::new(),
        }
    }

    async fn send_batch(&mut self) -> Result<(), BonsaiCompleteProofManagerError> {
        if self.ready_to_send_batch.is_empty() {
            return Ok(());
        }

        let proxy: ProxyContract<M> =
            ProxyContract::new(self.proxy_contract_address, self.ethers_client.clone());
        let proof_batch: Vec<Callback> = self
            .ready_to_send_batch
            .clone()
            .into_iter()
            .map(|complete_proof| complete_proof.ethereum_callback.into())
            .collect();

        info!("sending batch");
        let contract_call = proxy
            .invoke_callback(proof_batch)
            .gas(BONSAI_RELAY_GAS_LIMIT);
        let pending_tx =
            contract_call
                .send()
                .await
                .map_err(|e| BonsaiCompleteProofManagerError::Ethers {
                    source: Box::new(e),
                })?;
        let tx_hash = pending_tx.tx_hash();

        pending_tx
            .await
            .map_err(|e| BonsaiCompleteProofManagerError::Confirmation { source: e, tx_hash })?;

        for completed_proof in self.ready_to_send_batch.clone().into_iter() {
            self.storage
                .transition_proof_request(
                    completed_proof.bonsai_proof_id.clone(),
                    ProofRequestState::CompletedOnchain(tx_hash),
                )
                .await
                .map_err(|e| BonsaiCompleteProofManagerError::Storage {
                    source: e,
                    id: Some(completed_proof.bonsai_proof_id.clone()),
                })?;
        }

        self.ready_to_send_batch.clear();

        Ok(())
    }

    async fn process_new_complete_proof_requests(
        &mut self,
    ) -> Result<(), BonsaiCompleteProofManagerError> {
        let completed_proof_requests = self
            .storage
            .fetch_completed_bonsai_requests(None)
            .await
            .map_err(|e| BonsaiCompleteProofManagerError::Storage {
                source: e,
                id: None,
            })?;
        for request in completed_proof_requests.into_iter() {
            let completed_proof_request_handler = tokio::spawn(get_complete_proof(
                self.client.clone(),
                request.proof_request_id.clone(),
                request.callback_proof_request_event,
            ));
            self.futures_set.push(completed_proof_request_handler);

            self.storage
                .transition_proof_request(
                    request.proof_request_id.clone(),
                    ProofRequestState::PreparingOnchain,
                )
                .await
                .map_err(|e| BonsaiCompleteProofManagerError::Storage {
                    source: e,
                    id: Some(request.proof_request_id.clone()),
                })?;

            info!(?request.proof_request_id, "processing compeleted proof");
        }

        Ok(())
    }

    pub(crate) async fn handle_complete_proof_result(
        &mut self,
        completed_proof_result: Result<CompleteProof, CompleteProofError>,
    ) -> Result<(), BonsaiCompleteProofManagerError> {
        let completed_proof = completed_proof_result?;

        self.ready_to_send_batch.push(completed_proof.clone());
        if self.ready_to_send_batch.len() >= self.max_batch_size {
            self.send_batch_notifier.notify_one();
        }

        info!(?completed_proof, "proof added to batch");
        Ok(())
    }

    async fn reset_inflight_proof_requests(
        &mut self,
    ) -> Result<(), BonsaiCompleteProofManagerError> {
        let inflight_requests = self
            .storage
            .fetch_preparing_onchain_proof_requests(None)
            .await
            .map_err(|e| BonsaiCompleteProofManagerError::Storage {
                source: e,
                id: None,
            })?;

        for request in inflight_requests.into_iter() {
            self.storage
                .transition_proof_request(
                    request.proof_request_id.clone(),
                    ProofRequestState::Completed,
                )
                .await
                .map_err(|e| BonsaiCompleteProofManagerError::Storage {
                    source: e,
                    id: Some(request.proof_request_id.clone()),
                })?;
        }

        Ok(())
    }

    pub(crate) async fn step(&mut self) -> Result<(), BonsaiCompleteProofManagerError> {
        tokio::select! {
            Some(completed_proof_handle) = self.futures_set.next() => {
                let completed_proof_result = completed_proof_handle?;
                self.handle_complete_proof_result(completed_proof_result).await?
            }
            _ = self.new_complete_proofs_notifier.notified() => {
                self.process_new_complete_proof_requests().await?
            }

            _ = self.send_batch_interval.tick() => {
                self.send_batch_notifier.notify_one();
            }

            _ = self.send_batch_notifier.notified() => {
                self.send_batch().await?
            }
        }

        Ok(())
    }

    pub(crate) async fn run(mut self) -> Result<(), BonsaiCompleteProofManagerError> {
        self.reset_inflight_proof_requests().await?;
        self.process_new_complete_proof_requests().await?;

        loop {
            match self.step().await {
                e @ Err(BonsaiCompleteProofManagerError::JoinHandle(..)) => {
                    // if a task panics, just fail
                    return e;
                }
                Err(err) => {
                    // An error occurred processing the completed proof.
                    println!("error occurred managing pending proof requests: {:?}", err);
                    if let Some(proof_request_id) = err.get_proof_request_id() {
                        // Store the proof as new so that it can be retried.
                        //
                        // What do we do if this call to storage fails?
                        if let Err(err) = self
                            .storage
                            .transition_proof_request(proof_request_id, ProofRequestState::New)
                            .await
                        {
                            println!("failed to retry failed proof request: {:?}", err)
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
