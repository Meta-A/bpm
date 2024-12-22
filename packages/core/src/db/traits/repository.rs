#[async_trait::async_trait]
pub trait Repository<T, K> {
    async fn read_all(&self) -> Vec<T>;
    async fn read_by_key(&self, key: &K) -> Option<T>;
    async fn create(&self, document: &T);
    async fn update(&self, key: &K, document: &T);
    //async fn delete(&self, key: K) -> T;

    async fn exists_by_key(&self, key: &K) -> bool;
}
