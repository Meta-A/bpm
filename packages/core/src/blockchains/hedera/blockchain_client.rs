use std::{str::FromStr, time::Duration};

use crate::{
    blockchains::traits::{
        blockchain_information::BlockchainInformation, blockchain_reader::BlockchainReader,
        blockchain_writer::BlockchainWriter,
    },
    packages::{package::Package, package_builder::PackageBuilder},
};

use futures_util::TryStreamExt;
use hedera::{
    AccountId, Client, PrivateKey, TopicId, TopicMessageQuery, TopicMessageSubmitTransaction,
};
pub mod hedera_mirror {
    tonic::include_proto!("mirror");
}

use hedera_mirror::{
    com::hedera::mirror::api::proto::{
        consensus_service_client::ConsensusServiceClient, ConsensusTopicQuery,
        ConsensusTopicResponse,
    },
    proto::{Timestamp, TopicId as MirrorTopicId},
};

use log::{debug, info, trace};
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Streaming,
};

/**
 * Represents Hedera blockchain client, in our case we only use HCS
 */
#[derive(Clone)]
pub struct HederaBlockchainClient {
    blockchain_client: Client,
    packages_topic: TopicId,
}

impl HederaBlockchainClient {
    /**
     * Create new HederaBlockchainClient instance
     */
    pub fn new(package_topic_id: String) -> Result<Self, Box<dyn std::error::Error>> {
        debug!("Creating Hedera Blockchain Client using default parameters...");

        let blockchain_client = Client::for_testnet();

        let topic = TopicId::from_str(&package_topic_id)?;

        let client = Self {
            blockchain_client,
            packages_topic: topic,
        };

        debug!(
            "Done creating Hedera Blockchain Client from network address : {} !",
            client
                .blockchain_client
                .mirror_network()
                .first()
                .unwrap()
                .to_string()
        );

        Ok(client)
    }

    /**
     * Create new gRPC channel to HCS
     */
    async fn new_channel(&self) -> Result<Channel, Box<dyn std::error::Error>> {
        debug!("Establishing new HCS channel...");
        let tls = ClientTlsConfig::new().with_native_roots();

        let remote_url = format!("https://{}", self.get_net().to_string()); // We must prefix scheme

        let channel = Channel::from_shared(remote_url)?
            .tls_config(tls)?
            .connect()
            .await?;

        debug!("Done establishing new HCS channel !");

        Ok(channel)
    }

    /**
     * Subscribe to topic then return associated stream
     */
    async fn new_topic_subscription(
        &self,
        topic: TopicId,
    ) -> Result<Streaming<ConsensusTopicResponse>, Box<dyn std::error::Error>> {
        let query = ConsensusTopicQuery {
            topic_id: Some(MirrorTopicId {
                realm_num: i64::try_from(topic.realm)?,
                shard_num: i64::try_from(topic.shard)?,
                topic_num: i64::try_from(topic.num)?,
            }),
            consensus_start_time: Some(Timestamp {
                nanos: 0,
                seconds: 0,
            }),
            consensus_end_time: None,
            limit: 0,
        };

        let reading_channel = self.new_channel().await?;

        let mut mirror_client = ConsensusServiceClient::new(reading_channel.clone());

        let response = mirror_client.subscribe_topic(query).await?;

        let stream = response.into_inner();

        Ok(stream)
    }
}

impl BlockchainInformation for HederaBlockchainClient {
    fn get_net(&self) -> String {
        let networks = self.blockchain_client.mirror_network();

        let network = String::from(networks.first().unwrap());

        network
    }
}

#[async_trait::async_trait]
impl BlockchainWriter for HederaBlockchainClient {
    async fn submit_package(&self, package: &Package) -> Result<(), Box<dyn std::error::Error>> {
        info!("Submitting package {} to blockchain...", package.name);

        // TODO : handle hcs creds
        //let account_id = AccountId::from_str()?;
        //let private_key = PrivateKey::from_str()?;
        //
        //self.blockchain_client.set_operator(account_id, private_key);
        let serialized_package = serde_json::to_string(&package)?;

        trace!(
            "Sending following package as JSON to HCS : {}",
            serialized_package
        );

        let response = TopicMessageSubmitTransaction::new()
            .topic_id(self.packages_topic)
            .message(serialized_package)
            .execute(&self.blockchain_client)
            .await
            .unwrap();

        info!("Asking for receipt...");

        let receipt = response.get_receipt(&self.blockchain_client).await?;

        info!(
            "Package submission successful ! ( Seq number : {} )",
            receipt.topic_sequence_number
        );

        Ok(())
    }
}

impl BlockchainReader for HederaBlockchainClient {
    /**
     * Fetch packages in the blockchain remotely
     */
    async fn fetch_packages(&self) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
        info!("Fetching packages from Hedera blockchain...");

        let mut packages: Vec<Package> = Vec::new();

        let mut stream = self.new_topic_subscription(self.packages_topic).await?;

        debug!("Trying to fetch packages from HCS topic...");

        // How much time to wait before closing connection when expecting message
        const NEXT_MESSAGE_TIMEOUT: u64 = 1;

        while let Ok(result) =
            tokio::time::timeout(Duration::from_secs(NEXT_MESSAGE_TIMEOUT), stream.try_next()).await
        {
            let response = result.unwrap().unwrap();

            let raw_package = String::from_utf8(response.message)?;

            let package_parsing_result: Result<PackageBuilder, serde_json::Error> =
                PackageBuilder::from_json(raw_package);

            let mut builder = match package_parsing_result {
                Ok(builder) => builder,
                Err(_) => {
                    debug!(
                        "Package at location {} could not be parsed, skipping...",
                        response.sequence_number
                    );
                    continue;
                }
            };

            let package = builder.build();

            packages.push(package);
        }

        debug!("Done fetching packages from HCS topic !");

        info!(
            "Done fetching packages from Hedera blockchain ! ({} packages found)",
            packages.len()
        );

        Ok(packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchains::traits::blockchain_reader::BlockchainReader;
    /*
     * It should get network address
     */
    #[test]
    fn test_get_network_address() -> Result<(), Box<dyn std::error::Error>> {
        let expected_net_address = "testnet.mirrornode.hedera.com:443";
        let topic_id = "4991716";

        let client = HederaBlockchainClient::new(topic_id.to_string())?;

        assert_eq!(client.get_net(), expected_net_address);

        Ok(())
    }

    /**
     * It should retrieve packages from HCS
     */
    #[tokio::test]
    async fn test_get_packages() -> Result<(), Box<dyn std::error::Error>> {
        let topic_id = "4991716";

        let client = HederaBlockchainClient::new(topic_id.to_string())?;

        client.fetch_packages().await?;

        Ok(())
    }
}
