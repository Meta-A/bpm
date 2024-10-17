use std::error::Error;

use crate::{
    blockchains::abstractions::blockchain_client::BlockchainClient,
    packages::package_builder::PackageBuilder,
};

use futures_util::TryStreamExt;

pub mod proto {
    tonic::include_proto!("mod");
}

use log::debug;
use proto::{
    com::hedera::mirror::api::proto::{
        consensus_service_client::ConsensusServiceClient, ConsensusTopicQuery,
        ConsensusTopicResponse,
    },
    proto::{Timestamp, TopicId},
};
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Streaming,
};

/**
 * Represents Hedera blockchain client, in our case we only use HCS
 */
#[derive(Clone)]
pub struct HederaBlockchainClient {
    network_address: String,
    topic: Option<TopicId>,
    consensus_start_time: Option<Timestamp>,
    consensus_end_time: Option<Timestamp>,
}

impl From<String> for HederaBlockchainClient {
    /**
     * Creates new HederaBlockchainClient instance using net address
     */
    fn from(network_address: String) -> Self {
        debug!(
            "Creating Hedera Blockchain Client from network address : {}...",
            network_address
        );
        // By default start from beginning
        let consensus_start_time = Some(Timestamp {
            seconds: 0,
            nanos: 0,
        });

        let consensus_end_time = None;

        let client = Self {
            network_address,
            topic: None,
            consensus_start_time,
            consensus_end_time,
        };

        debug!(
            "Done creating Hedera Blockchain Client from network address : {} !",
            client.network_address
        );
        client
    }
}

impl BlockchainClient for HederaBlockchainClient {
    fn get_net(&self) -> &String {
        &self.network_address
    }

    async fn get_packages(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.get_net());
        let package_builder = PackageBuilder::new();

        let mut stream = self.subscribe_topic().await?;

        while let Some(tm) = stream.try_next().await? {
            let message = String::from_utf8(tm.message)?;

            println!("{}", message);
        }

        Ok(())
    }
}

impl HederaBlockchainClient {
    /**
     * Attaches HCS topic to client
     */
    pub fn with_topic(&mut self, topic_num: i64, shard_num: i64, realm_num: i64) -> &Self {
        let target_topic = TopicId {
            topic_num,
            shard_num,
            realm_num,
        };

        self.topic = Some(target_topic);

        self
    }

    /**
     * Subscribes to topic
     */
    pub async fn subscribe_topic(
        &self,
    ) -> Result<Streaming<ConsensusTopicResponse>, Box<dyn Error>> {
        let query = ConsensusTopicQuery {
            topic_id: self.topic,
            consensus_start_time: self.consensus_start_time,
            consensus_end_time: self.consensus_end_time,
            limit: 0,
        };

        let tls = ClientTlsConfig::new().with_native_roots();

        let channel = Channel::from_shared(self.network_address.clone())?
            .tls_config(tls)?
            .connect()
            .await?;

        let mut client = ConsensusServiceClient::new(channel);

        let response = client.subscribe_topic(query).await?;

        let stream = response.into_inner();

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
     * It should create client from network address
     */
    #[test]
    fn test_create_client_from_network_address() {
        let expected_net_address = "https://foobar.com:443";
        let client = HederaBlockchainClient::from(expected_net_address.to_string());

        assert_eq!(client.network_address, expected_net_address);
    }

    /**
     * It should attach topic to client
     */
    #[test]
    fn test_attach_topic_to_client() {
        let topic_num_mock: i64 = 4000000;
        let shard_num_mock: i64 = 0;
        let real_num_mock: i64 = 0;

        let expected_topic = TopicId {
            topic_num: topic_num_mock,
            shard_num: shard_num_mock,
            realm_num: real_num_mock,
        };

        let net_address_mock = "https://foobar.com:443";
        let mut client = HederaBlockchainClient::from(net_address_mock.to_string());

        let client_with_topic = client.with_topic(
            expected_topic.topic_num,
            expected_topic.shard_num,
            expected_topic.realm_num,
        );

        assert_eq!(client_with_topic.topic.unwrap(), expected_topic);
    }

    /**
     * It should get network address
     */
    #[test]
    fn test_get_network_address() {
        let expected_net_address = "https://foobar.com:443";
        let client = HederaBlockchainClient::from(expected_net_address.to_string());

        assert_eq!(client.get_net(), expected_net_address);
    }
}
