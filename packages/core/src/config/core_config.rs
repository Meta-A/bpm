/**
 * Represents application's configuration
 */
#[serde_with::skip_serializing_none]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CoreConfig {
    pub proxy: Option<String>,
}
