use std::{
    error::Error,
    fs::{create_dir_all, File},
    io::{BufWriter, Error as IOError, ErrorKind, Write},
    path::PathBuf,
};

use config::{Config, FileFormat};
use log::debug;

use super::app_config::AppConfig;

const DEFAULT_CONFIG: AppConfig = AppConfig { proxy: None };

/**
 * Configuration manager
 *
 * Manages reading / writing values to config file
 */
pub struct ConfigManager {
    pub path: PathBuf,
}

impl ConfigManager {
    /**
     * Write default config values to given file
     */
    fn write_default_config(file: &File) -> Result<(), IOError> {
        debug!("Writing default config values...");

        let mut writer = BufWriter::new(file);

        serde_json::to_writer(&mut writer, &DEFAULT_CONFIG)?;

        writer.flush()?;

        debug!("Done writing default config values !");

        Ok(())
    }

    /**
     * Create config file at given path
     */
    fn create_config_file(path: &PathBuf) -> Result<File, IOError> {
        let path_display = path.display().to_string();

        debug!("Creating config file at {}...", path_display);

        let mut dir_path_components = path.components();

        dir_path_components.next_back();

        let dir_path = dir_path_components.as_path();

        create_dir_all(dir_path)?;

        let file = File::create_new(path.as_os_str().to_str().unwrap())?;

        ConfigManager::write_default_config(&file)?;

        debug!("Done writing config file at {} !", path_display);

        Ok(file)
    }

    /**
     * Load config
     */
    pub fn load(&self) -> Result<Config, IOError> {
        debug!("Loading BBPM config...");

        let config = Config::builder()
            .add_source(config::File::new(
                self.path.as_path().as_os_str().to_str().unwrap(),
                FileFormat::Json,
            ))
            .add_source(config::Environment::with_prefix("BBPM"))
            .build()
            .unwrap();

        debug!("Done loading BBPM config !");

        Ok(config)
    }

    /**
     * Instantiates ConfigManager while making sure config file exists
     */
    pub fn from(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        debug!(
            "Building ConfigManager using path {}...",
            path.display().to_string()
        );

        ConfigManager::create_config_file(path).unwrap_or_else(|error| {
            if error.kind() == ErrorKind::AlreadyExists {
                debug!("Configuration file already exists cannot create, skipping...");
                let existing_file = File::open(&path).unwrap();

                existing_file
            } else {
                panic!("Could not create configuration file: {error:?}");
            }
        });

        let manager = ConfigManager { path: path.clone() };

        debug!(
            "Done building ConfigManager using path {} !",
            path.display().to_string()
        );

        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    /**
     * It should write default config values to file
     */
    #[test]
    fn test_write_default_config_values() {
        let test_dir = TempDir::new().unwrap();

        let test_file_path = test_dir.path().join("test.json");

        let file = File::create_new(test_file_path).unwrap();

        ConfigManager::write_default_config(&file).unwrap();
    }

    /**
     * It should create config file at given location
     */
    #[test]
    fn test_config_file_creation() {
        let test_dir = TempDir::new().unwrap();

        let expected_config_file_path = test_dir.path().join("config.json");

        ConfigManager::create_config_file(&expected_config_file_path).unwrap();

        assert_eq!(expected_config_file_path.exists(), true);
    }

    /**
     * It should create ConfigManager from path
     */
    #[test]
    fn test_create_manager_from_path() {
        let test_dir = TempDir::new().unwrap();

        let expected_config_file_path = &test_dir.into_path().join("config.json");

        let config_manager = ConfigManager::from(expected_config_file_path).unwrap();

        assert_eq!(
            config_manager.path.as_os_str().to_str().unwrap(),
            expected_config_file_path.as_os_str().to_str().unwrap()
        );
    }

    /**
     * It should use config file if it already exists using ConfigManager from path
     */
    #[test]
    fn test_create_manager_from_path_with_config_existing_file() {
        let test_dir = TempDir::new().unwrap();

        let expected_config_file_path = &test_dir.into_path().join("config.json");

        let mut config_manager = ConfigManager::from(expected_config_file_path).unwrap();

        // Config file is now created, try to load it once again

        config_manager = ConfigManager::from(expected_config_file_path).unwrap();

        assert_eq!(
            config_manager.path.as_os_str().to_str().unwrap(),
            expected_config_file_path.as_os_str().to_str().unwrap()
        );
    }

    /**
     * It should panic while creating ConfigManager when unhandled error
     */
    #[test]
    #[should_panic]
    fn test_create_manager_unhandled_error_panic() {
        let test_dir = TempDir::new().unwrap();

        let test_dir_path = test_dir.path();

        // We set permissions to read only on purpose so that ::from will try to write and raise
        // Error
        let mut perms = fs::metadata(test_dir_path).unwrap().permissions();

        perms.set_readonly(true);

        fs::set_permissions(test_dir_path, perms).unwrap();

        let expected_config_file_path = &test_dir.into_path().join("");

        ConfigManager::from(expected_config_file_path).unwrap();
    }

    /**
     * It shoud load config without panicking
     */
    #[test]
    fn test_load_config() {
        let test_dir = TempDir::new().unwrap();

        let expected_config_file_path = &test_dir.into_path().join("config.json");

        let config_manager = ConfigManager::from(expected_config_file_path).unwrap();

        config_manager.load().unwrap();
    }
}
