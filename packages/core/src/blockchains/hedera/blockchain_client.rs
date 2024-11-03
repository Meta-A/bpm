use std::{env, str::FromStr, time::Duration};

use crate::blockchains::blockchain::{BlockchainClient, BlockchainIO};

use futures_util::TryStreamExt;
use hedera::{AccountId, Client, PrivateKey, TopicId, TopicMessageSubmitTransaction};
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

use tokio::sync::mpsc::Sender;

use log::{debug, trace};
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Streaming,
};

struct HederaBlockchainIO {
    network: String,
    last_sync: u64,
    packages_topic: TopicId,
    hedera_client: Client,
}

impl HederaBlockchainIO {
    /**
     * Create new gRPC channel to HCS
     */
    async fn new_channel(&self) -> Result<Channel, Box<dyn std::error::Error>> {
        debug!("Establishing new HCS channel...");
        let tls = ClientTlsConfig::new().with_native_roots();

        let remote_url = format!("https://{}", self.network.to_string()); // We must prefix scheme

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
        start_timestamp: u64,
    ) -> Result<Streaming<ConsensusTopicResponse>, Box<dyn std::error::Error>> {
        debug!("Creating new topic subscription...");
        let query = ConsensusTopicQuery {
            topic_id: Some(MirrorTopicId {
                realm_num: i64::try_from(topic.realm)?,
                shard_num: i64::try_from(topic.shard)?,
                topic_num: i64::try_from(topic.num)?,
            }),
            consensus_start_time: Some(Timestamp {
                nanos: 0,
                // TODO : not sure about that, handle it better way
                seconds: i64::from_ne_bytes(start_timestamp.to_ne_bytes()),
            }),
            consensus_end_time: None,
            limit: 0,
        };

        let reading_channel = self.new_channel().await?;

        let mut mirror_client = ConsensusServiceClient::new(reading_channel.clone());

        let response = mirror_client.subscribe_topic(query).await?;

        let stream = response.into_inner();

        debug!("Done creating new topic subscription !");
        Ok(stream)
    }
}

#[async_trait::async_trait]
impl BlockchainIO for HederaBlockchainIO {
    async fn write(&self, data: &[u8]) {
        TopicMessageSubmitTransaction::new()
            .topic_id(self.packages_topic)
            .message(data)
            .execute(&self.hedera_client)
            .await
            .unwrap();
    }

    async fn read(&self, tx_data: &Sender<Vec<u8>>) {
        let mut stream = self
            .new_topic_subscription(self.packages_topic, self.last_sync)
            .await
            .unwrap();

        const NEXT_MESSAGE_TIMEOUT: u64 = 1;
        while let Ok(result) =
            tokio::time::timeout(Duration::from_secs(NEXT_MESSAGE_TIMEOUT), stream.try_next()).await
        {
            trace!("Sending to channel...");
            let response = result.unwrap().unwrap();

            let buf: Vec<u8> = Vec::from(response.message.as_slice());

            tx_data.send(buf).await.unwrap();
            trace!("Done sending to channel !");
        }
    }
}

#[derive(Debug, Clone)]
pub struct HederaBlockchain {
    blockchain_client: Client,
    packages_topic: TopicId,
    last_sync: u64,
}

#[async_trait::async_trait]
impl BlockchainClient for HederaBlockchain {
    fn get_network(&self) -> String {
        let networks = self.blockchain_client.mirror_network();

        let network = String::from(networks.first().unwrap());

        network
    }

    fn get_last_sync(&self) -> u64 {
        self.last_sync
    }

    fn get_label(&self) -> String {
        String::from("hedera")
    }

    fn create_io(&self) -> Box<dyn BlockchainIO> {
        let hedera_io = HederaBlockchainIO {
            network: self.get_network(),
            last_sync: 0,
            packages_topic: self.packages_topic,
            hedera_client: self.blockchain_client.clone(),
        };

        Box::new(hedera_io)
    }
}

impl From<&str> for HederaBlockchain {
    fn from(package_topic_id: &str) -> Self {
        debug!("Creating Hedera Blockchain Client using default parameters...");

        let blockchain_client = Client::for_testnet();

        // TODO : temporary, use config manager
        let debug_account = env::var("BPM_ACCOUNT").unwrap_or(String::from(""));
        let debug_key = env::var("BPM_KEY").unwrap_or(String::from(""));

        if debug_account != "" && debug_key != "" {
            let account_id = AccountId::from_str(debug_account.as_str()).unwrap();
            let private_key = PrivateKey::from_str(debug_key.as_str()).unwrap();
            blockchain_client.set_operator(account_id, private_key);
        }

        let topic = TopicId::from_str(&package_topic_id).unwrap();

        let client = Self {
            blockchain_client,
            packages_topic: topic,
            last_sync: 0,
        };

        let net_addr = client
            .blockchain_client
            .mirror_network()
            .first()
            .unwrap()
            .to_string();

        debug!(
            "Done creating Hedera Blockchain Client using network address : {} !",
            net_addr
        );

        client
    }
}
