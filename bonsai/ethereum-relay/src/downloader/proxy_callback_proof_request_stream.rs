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

use anyhow::Result;
use bonsai_proxy_contract::CallbackRequestFilter;
use ethers::{
    providers::{Middleware, SubscriptionStream, PubsubClient, Ws, Provider},
    types::{Address, BlockNumber, Log}, prelude::{signer::SignerMiddlewareError, k256::ecdsa::SigningKey},
};
use ethers_signers::Wallet;
use futures::{Stream, StreamExt};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, };

use super::{block_history, block_history::State};
use crate::{api::error::Error, downloader::event_processor::EventProcessor, EthersClientConfig};

#[derive(Debug)]
pub(crate) struct ProxyCallbackProofRequestStream<
    EP: EventProcessor<Event = CallbackRequestFilter> + Sync + Send,
> {
    client_config: EthersClientConfig,
    proxy_contract_address: Address,
    event_processor: EP,
}

impl<EP: EventProcessor<Event = CallbackRequestFilter> + Sync + Send>
    ProxyCallbackProofRequestStream<EP>
{
    pub(crate) fn new(
        client_config: EthersClientConfig,
        proxy_contract_address: Address,
        event_processor: EP,
    ) -> ProxyCallbackProofRequestStream<EP> {
        Self {
            client_config,
            proxy_contract_address,
            event_processor,
        }
    }

    pub(crate) async fn run(self) -> Result<(), Error> {
        const EVENT_NAME: &str = "CallbackRequest(address,bytes32,bytes,address,bytes4,uint64)";

        let filter = ethers::types::Filter::new()
            .address(self.proxy_contract_address)
            .event(EVENT_NAME);
        let client = self.client_config.get_client().await?;
        let last_processed_block_number = client.get_block_number().await?;
        let last_processed_block = BlockNumber::Number(last_processed_block_number);
        let mut state = State {
            client_config: self.client_config.clone(),
            client,
            recreate_client: false,
            last_processed_block: last_processed_block_number,
            filter,
            latest_block: last_processed_block_number,
            from: last_processed_block,
            to: last_processed_block,
        };

        loop {
            state = self.recreate_client(state.clone()).await?;
            let state2 = state.clone();
            // Recover block delay
            {
                let (tx, rx) = tokio::sync::mpsc::channel(100);
                let recover_lost_blocks_handler = tokio::spawn(async move {
                    block_history::recover_lost_blocks(state2, tx.clone()).await
                });
                self.process_logs(ReceiverStream::new(rx)).await;
                match recover_lost_blocks_handler.await {
                    Ok(Ok(new_state)) => {
                        state = new_state;
                        debug!("Successfully recovered block delay.");
                    }
                    Ok(Err(error)) => {
                        error!(?error, "Failed to recover block delay.");
                    }
                    Err(error) => {
                        error!(?error, "Tokio Task `recover_last_blocks_handler` failed.");
                    }
                }
            }
            state = State {
                filter: state
                    .filter
                    .clone()
                    .from_block(BlockNumber::Number(state.last_processed_block)),
                ..state.clone()
            };
            let logs = state.client.subscribe_logs(&state.filter).await;
            self.match_logs(state.clone(), logs).await;
        }
    }

    async fn recreate_client(&self, state: State) -> Result<State, Error> {
        let state = if state.recreate_client {
            debug!("Recreating client.");
            state.recreate_client().await?
        } else {
            state
        };
        Ok(state)
    }

    async fn process_logs(&self, stream: impl Stream<Item = Log>) {
        tokio::pin!(stream);
        while let Some(log) = stream.next().await {
            let parsed_event: Result<CallbackRequestFilter, _> = ethers::contract::parse_log(log);
            match parsed_event {
                Ok(event) => {
                    if let Err(error) = self.event_processor.process_event(event).await {
                        error!(?error, "Error processing event");
                    }
                }
                Err(error) => error!(?error, "Error parsing log"),
            }
        }
    }

    async fn match_logs(
        &self,
        state: State,
        logs: Result<
            SubscriptionStream<'_, impl PubsubClient, Log>,
            SignerMiddlewareError<Provider<Ws>, Wallet<SigningKey>>,
        >,
    ) -> State {
        match logs {
            Ok(logs) => {
                debug!("Successfully subscribed to logs");
                self.process_logs(logs).await;
                state
            }
            Err(error) => {
                error!(?error, "Failed to subscribe to logs");
                State {
                    recreate_client: true,
                    ..state
                }
            }
        }
    }
}
