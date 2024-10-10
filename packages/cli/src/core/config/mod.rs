pub mod app_config;
pub mod manager;

use std::path::PathBuf;

use home::home_dir;
use log::debug;
use manager::ConfigManager;

pub fn init_config(path: &PathBuf) -> ConfigManager {
    let path_display = path.display().to_string();

    debug!(
        "Initializing config file, provided location : {}",
        path_display
    );

    let config_file_path = path.join(".bbpm").join("config.json");

    let config_manager = ConfigManager::from(&config_file_path).unwrap();

    debug!(
        "Done initializing config file using location {} !",
        path_display
    );

    config_manager
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    /**
     * It should initialize config file
     */
    #[test]
    fn test_init_config() {
        let test_dir = TempDir::new().unwrap();

        let expected_config_file_path = test_dir.path().join(".bbpm").join("config.json");

        init_config(&test_dir.into_path());

        assert_eq!(expected_config_file_path.exists(), true);
    }
}
