/**
 * Represents application's configuration
 */
#[serde_with::skip_serializing_none]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub proxy: Option<String>,
}
