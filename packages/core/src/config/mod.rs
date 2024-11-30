pub mod core_config;
pub mod manager;

use std::path::PathBuf;

use log::debug;
use manager::ConfigManager;

const CONFIG_DIR_NAME: &str = ".bpm";

/**
 * Initialize configuration
 */
pub fn init_config(path: &PathBuf) -> ConfigManager {
    let path_display = path.display().to_string();

    debug!(
        "Initializing config file, provided location : {}",
        path_display
    );

    let config_path = path.join(CONFIG_DIR_NAME);

    let config_manager = ConfigManager::from(&config_path);

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

    #[test]
    fn test_init_config() {
        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path().to_path_buf();

        let config_manager = init_config(&test_dir_path);

        let expected_dir_path = test_dir.path().join(CONFIG_DIR_NAME);

        assert_eq!(config_manager.get_path(), expected_dir_path);
    }
}
