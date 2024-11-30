#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use tempfile::TempDir;

    use crate::db::client::DbClient;

    pub fn create_test_db() -> Arc<DbClient> {
        let db_dir = "db";

        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().join(db_dir);

        let db_client = Arc::new(DbClient::from(&test_dir_path));

        db_client
    }
}
