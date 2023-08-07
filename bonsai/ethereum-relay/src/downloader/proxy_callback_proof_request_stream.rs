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
use bonsai_proxy_contract::CallbackRequestFilter;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    prelude::{signer::SignerMiddlewareError, *},
    providers::{Middleware, Provider, PubsubClient, SubscriptionStream, Ws},
    types::{Address, Log},
};
use futures::StreamExt;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, warn};

use super::block_history;
use crate::{api::error::Error, downloader::event_processor::EventProcessor, EthersClientConfig};

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
        let mut client = self.client_config.get_client().await?;
        let mut recreate_client = false;
        let mut last_processed_block = client.get_block_number().await?;

        loop {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            last_processed_block = self
                .recover_lost_blocks(
                    &mut client,
                    last_processed_block,
                    &mut recreate_client,
                    tx.clone(),
                )
                .await?;
            self.recreate_client(&mut client, &mut recreate_client)
                .await?;
            let logs = client.subscribe_logs(&filter).await;
            self.match_logs(logs, &mut recreate_client).await;
        }
    }

    async fn recreate_client(
        &self,
        client: &mut SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>,
        recreate_flag: &mut bool,
    ) -> Result<(), Error> {
        if *recreate_flag {
            debug!("Recreating client.");
            *client = self.client_config.get_client_with_reconnects().await?;
            *recreate_flag = false;
        };
        Ok(())
    }

    async fn process_logs(
        &self,
        mut subscription_stream: SubscriptionStream<'_, impl PubsubClient, Log>,
    ) -> Result<(), Error> {
        while let Some(log) = subscription_stream.next().await {
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
        Ok(())
    }

    async fn match_logs(
        &self,
        logs: Result<
            SubscriptionStream<'_, impl PubsubClient, Log>,
            SignerMiddlewareError<Provider<Ws>, Wallet<SigningKey>>,
        >,
        recreate_client_flag: &mut bool,
    ) {
        match logs {
            Ok(logs) => {
                debug!("Successfully subscribed to logs");
                self.process_logs(logs).await.map_or_else(
                    |error| error!(?error, "Failed to process logs"),
                    |_| {
                        warn!("Proxy stream ended, reconnecting...");
                        *recreate_client_flag = true
                    },
                )
            }
            Err(error) => {
                error!(?error, "Failed to subscribe to logs");
                *recreate_client_flag = true;
            }
        }
    }

    async fn recover_lost_blocks(
        &self,
        client: &mut SignerMiddleware<Provider<Ws>, Wallet<SigningKey>>,
        last_processed_block: BlockNumber,
        recreate_client_flag: &mut bool,
        sender: Sender<Log>,
    ) -> U64 {
        let last_block = match block_history::get_latest_block(client.provider()).await {
            Ok(Some(block)) => block,
            Ok(None) => {
                warn!("Latest block is not available, retrying...");
                *recreate_client_flag = true;
                return last_processed_block;
            }
            Err(error) => {
                error!(?error, "Failed to get latest block number");
                *recreate_client_flag = true;
                return last_processed_block;
            }
        };
        block_history::recover_delay(
            self.client_config.clone(),
            last_processed_block,
            last_block,
            sender,
        )
        .await
    }
}
