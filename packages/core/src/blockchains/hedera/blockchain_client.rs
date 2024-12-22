use crate::blockchains::blockchain::{BlockchainClient, BlockchainIO};
use crate::blockchains::errors::blockchain_error::BlockchainError;
use std::convert::TryFrom;
use std::{env, str::FromStr, sync::Arc, time::Duration};

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

use tokio::sync::{mpsc::Sender, Mutex};

use log::{debug, trace};
use tonic::{
    transport::{Channel, ClientTlsConfig},
    Streaming,
};

#[cfg(test)]
use mockall::automock;

#[derive(Debug, Clone)]
struct HederaBlockchainIO {
    packages_topic: TopicId,
    hedera_client: Client,
}

#[cfg_attr(test, automock)]
impl HederaBlockchainIO {
    /**
     * Create new gRPC channel to HCS
     */
    async fn new_channel(&self) -> Result<Channel, BlockchainError> {
        debug!("Establishing new HCS channel...");

        let networks = self.hedera_client.mirror_network();

        let network = String::from(networks.first().unwrap());

        let tls = ClientTlsConfig::new().with_native_roots();

        let remote_url = format!("https://{}", network.to_string()); // We must prefix scheme

        let channel = Channel::from_shared(remote_url)
            .map_err(|_| BlockchainError::ConnectionConfig)?
            .tls_config(tls)
            .map_err(|_| BlockchainError::ConnectionConfig)?
            .connect()
            .await
            .map_err(|_| BlockchainError::ConnectionFailure)?;

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
    ) -> Result<Streaming<ConsensusTopicResponse>, BlockchainError> {
        debug!("Creating new topic subscription...");

        let query = ConsensusTopicQuery {
            topic_id: Some(MirrorTopicId {
                realm_num: i64::try_from(topic.realm)
                    .expect("Could not convert topic realm to i64"),
                shard_num: i64::try_from(topic.shard).expect("Could not convert shard to i64"),
                topic_num: i64::try_from(topic.num).expect("Could not convert topic num to i64"),
            }),
            consensus_start_time: Some(Timestamp {
                nanos: 0,
                seconds: i64::try_from(start_timestamp)
                    .expect("Could not convert start time seconds to i64"),
            }),
            consensus_end_time: None,
            limit: 0,
        };

        let reading_channel = self.new_channel().await?;

        let mut mirror_client = ConsensusServiceClient::new(reading_channel.clone());

        const TIMEOUT: u64 = 1;
        let response = tokio::time::timeout(
            Duration::from_secs(TIMEOUT),
            mirror_client.subscribe_topic(query),
        )
        .await
        .map_err(|_| BlockchainError::NoPackagesData)?;

        let stream = response.unwrap().into_inner();
        debug!("Done creating new topic subscription !");
        Ok(stream)
    }
}

#[async_trait::async_trait]
impl BlockchainIO for HederaBlockchainIO {
    /**
     * Write to HCS
     */
    async fn write(&self, data: &[u8]) {
        TopicMessageSubmitTransaction::new()
            .topic_id(self.packages_topic)
            .message(data)
            .execute(&self.hedera_client)
            .await
            .unwrap();
    }

    /**
     * Read from HCS
     */
    async fn read(&self, tx_data: &Sender<Result<Vec<u8>, BlockchainError>>, last_sync: &u64) {
        let stream_res = self
            .new_topic_subscription(self.packages_topic, *last_sync)
            .await;

        let mut stream = match stream_res {
            Ok(stream) => stream,
            Err(e) => {
                tx_data.send(Err(e)).await.unwrap();
                return ();
            }
        };

        const NEXT_MESSAGE_TIMEOUT: u64 = 1;

        while let Ok(result) =
            tokio::time::timeout(Duration::from_secs(NEXT_MESSAGE_TIMEOUT), stream.try_next()).await
        {
            trace!("Sending to channel...");
            let response = result.unwrap().unwrap();

            let buf: Vec<u8> = Vec::from(response.message.as_slice());

            tx_data.send(Ok(buf)).await.unwrap();
            trace!("Done sending to channel !");
        }
    }
}

impl From<&str> for HederaBlockchainIO {
    fn from(package_topic_id: &str) -> Self {
        // TODO : temporary, use config manager
        let debug_account = env::var("BPM_ACCOUNT").unwrap_or(String::from(""));
        let debug_key = env::var("BPM_KEY").unwrap_or(String::from(""));

        let blockchain_client = Client::for_testnet();

        if debug_account != "" && debug_key != "" {
            let account_id = AccountId::from_str(debug_account.as_str()).unwrap();
            let private_key = PrivateKey::from_str(debug_key.as_str()).unwrap();
            blockchain_client.set_operator(account_id, private_key);
        }

        let topic = TopicId::from_str(&package_topic_id).unwrap();

        let instance = Self {
            hedera_client: blockchain_client,
            packages_topic: topic,
        };

        instance
    }
}

#[derive(Debug)]
pub struct HederaBlockchain {
    hedera_io: Arc<Box<dyn BlockchainIO>>,
    last_sync: Arc<Mutex<u64>>,
}

impl HederaBlockchain {
    pub fn new(hedera_io: Box<dyn BlockchainIO>) -> Self {
        let instance = Self {
            hedera_io: Arc::new(hedera_io),
            last_sync: Arc::new(Mutex::new(0)),
        };

        instance
    }
}

#[async_trait::async_trait]
#[cfg_attr(test, automock)]
impl BlockchainClient for HederaBlockchain {
    /**
     * Get blockchain label
     */
    fn get_label(&self) -> String {
        String::from("hedera")
    }

    /**
     * Create HCS IO
     */
    async fn create_io(&self) -> Arc<Box<dyn BlockchainIO>> {
        Arc::clone(&self.hedera_io)
    }

    /**
     * Get last sync
     */
    async fn get_last_sync(&self) -> u64 {
        let last_sync = self.last_sync.lock().await;
        *last_sync
    }

    /**
     * Set last sync
     */
    async fn set_last_sync(&self, last_sync: u64) {
        let mut last_sync_lock = self.last_sync.lock().await;

        *last_sync_lock = last_sync;
    }
}

impl From<&str> for HederaBlockchain {
    /**
     * Build from HCS topic ID
     */
    fn from(package_topic_id: &str) -> Self {
        debug!("Creating Hedera Blockchain Client using default parameters...");
        let default_last_sync = 0;

        let hedera_io = Box::new(HederaBlockchainIO::from(package_topic_id));

        let net_addr = hedera_io
            .hedera_client
            .mirror_network()
            .first()
            .unwrap()
            .to_string();

        let client = Self {
            hedera_io: Arc::new(hedera_io),
            last_sync: Arc::new(Mutex::new(default_last_sync)),
        };

        debug!(
            "Done creating Hedera Blockchain Client using network address : {} !",
            net_addr
        );

        client
    }
}

#[cfg(test)]
mod tests {
    use crate::blockchains::blockchain::{BlockchainClient, BlockchainIO, MockBlockchainIO};

    use super::HederaBlockchain;

    /**
     * It should get label
     */
    #[tokio::test]
    async fn test_should_get_label() {
        let hedera_io_mock = MockBlockchainIO::default();

        let hedera_io: Box<dyn BlockchainIO> = Box::new(hedera_io_mock);

        let blockchain_client = HederaBlockchain::new(hedera_io);

        let expected_label = String::from("hedera");

        let current_label = blockchain_client.get_label();
        assert_eq!(current_label, expected_label);
    }

    /**
     * It should set last sync
     */
    #[tokio::test]
    async fn test_should_set_last_sync() {
        let mut hedera_io_mock = MockBlockchainIO::default();

        hedera_io_mock
            .expect_read()
            .returning(|_, _| Box::pin(async {}));

        hedera_io_mock
            .expect_write()
            .returning(|_| Box::pin(async {}));

        let hedera_io: Box<dyn BlockchainIO> = Box::new(hedera_io_mock);

        let blockchain_client = HederaBlockchain::new(hedera_io);

        let expected_last_sync = 123;

        blockchain_client.set_last_sync(expected_last_sync).await;

        let current_last_sync = blockchain_client.get_last_sync().await;
        assert_eq!(current_last_sync, expected_last_sync);
    }

    /**
     * It should create IO
     */
    #[tokio::test]
    async fn test_should_create_io() {
        let mut hedera_io_mock = MockBlockchainIO::default();

        hedera_io_mock
            .expect_read()
            .returning(|_, _| Box::pin(async {}));

        hedera_io_mock
            .expect_write()
            .returning(|_| Box::pin(async {}));

        let hedera_io: Box<dyn BlockchainIO> = Box::new(hedera_io_mock);

        let blockchain_client = HederaBlockchain::new(hedera_io);

        let io = blockchain_client.create_io().await;
    }
}
