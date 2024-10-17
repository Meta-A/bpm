use std::error::Error;

use crate::{
    blockchains::abstractions::blockchain_client::BlockchainClient,
    packages::package_builder::PackageBuilder,
};

use futures_util::TryStreamExt;

pub mod proto {
    tonic::include_proto!("mod");
}

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
        // By default start from beginning
        let consensus_start_time = Some(Timestamp {
            seconds: 0,
            nanos: 0,
        });

        let consensus_end_time = None;

        Self {
            network_address,
            topic: None,
            consensus_start_time,
            consensus_end_time,
        }
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
