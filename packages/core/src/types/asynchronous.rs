use tokio::sync::Mutex;

pub type AsyncMutex<T> = Mutex<T>;
