use std::{
    fs::{self, create_dir_all, File},
    io::{BufWriter, Error as IOError, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

use ed25519::{
    pkcs8::{spki::der::pem::LineEnding, DecodePrivateKey, EncodePrivateKey},
    signature::rand_core::OsRng,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use log::debug;

use super::core_config::CoreConfig;

const DEFAULT_CONFIG: CoreConfig = CoreConfig { proxy: None };

const PRIVATE_KEY_FILENAME: &str = "key.pem";

const DB_DIR_NAME: &str = "db";

/**
 * Configuration manager
 *
 * Manages BPM config ( config file, key generation.... )
 */
pub struct ConfigManager {
    path: PathBuf,
}

impl ConfigManager {
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
     * Generate new key
     */
    fn generate_key() -> SigningKey {
        debug!("Generating new key...");

        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);

        debug!("Done generating new key !");

        signing_key
    }

    /**
     * Write key file
     */
    fn write_key_file(key_path: &PathBuf) -> Result<File, Box<dyn std::error::Error>> {
        debug!("Writing key file...");

        // Generate new key
        let maintainer_signing_key = ConfigManager::generate_key();

        let encoded_private_key = maintainer_signing_key.to_pkcs8_pem(LineEnding::LF)?;

        let mut key_file = File::create(&key_path)?;

        let mut key_file_permissions = key_file.metadata().unwrap().permissions();

        key_file_permissions.set_mode(0o400);
        fs::set_permissions(&key_path, key_file_permissions)?;

        key_file.write_all(encoded_private_key.as_bytes())?;

        debug!("Done writing key file !");

        Ok(key_file)
    }

    ///**
    // * Load config
    // */
    //pub fn load(&self) -> Result<Config, IOError> {
    //    debug!("Loading BPM config...");
    //
    //    let config = Config::builder()
    //        .add_source(config::File::new(
    //            self.path.as_path().as_os_str().to_str().unwrap(),
    //            FileFormat::Json,
    //        ))
    //        .add_source(config::Environment::with_prefix("BBPM"))
    //        .build()
    //        .unwrap();
    //
    //    debug!("Done loading BPM config !");
    //
    //    Ok(config)
    //}

    /**
     * Handle initializing config for first time
     */
    fn init_config(directory_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Initializing config directory...");

        let config_exists = fs::exists(directory_path)?;

        if config_exists == false {
            debug!("Creating default config file...");
            let config_file_path = directory_path.join("config.json");

            ConfigManager::create_config_file(&config_file_path)?;
            debug!("Done creating default config file !");

            let key_path = directory_path.join(PRIVATE_KEY_FILENAME);
            ConfigManager::write_key_file(&key_path)?;

            debug!("Done initializing config directory !");
        }

        Ok(())
    }

    /**
     * Get config dir path
     */
    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    /**
     * Get DB config path
     */
    pub fn get_db_path(&self) -> PathBuf {
        self.path.join(DB_DIR_NAME)
    }

    /**
     * Retrieve signing key
     */
    pub fn get_signing_key(&self) -> Result<SigningKey, Box<dyn std::error::Error>> {
        debug!("Retrieving signing key...");

        let key_file_path = self.path.join(PRIVATE_KEY_FILENAME);

        let key_buf = fs::read_to_string(key_file_path)?;

        let key = SigningKey::from_pkcs8_pem(key_buf.as_str())?;

        debug!("Done retrieving signing key !");

        Ok(key)
    }

    /**
     * Retrieve verifying key
     */
    pub fn get_verifying_key(&self) -> Result<VerifyingKey, Box<dyn std::error::Error>> {
        debug!("Retrieving verifying key...");

        let signing_key = self.get_signing_key()?;

        let verifying_key = signing_key.verifying_key();

        debug!("Done retrieving verifying key !");

        Ok(verifying_key)
    }
}

impl From<&PathBuf> for ConfigManager {
    /**
     * Instantiate ConfigManager while making sure config file exists
     */
    fn from(directory_path: &PathBuf) -> Self {
        debug!(
            "Building ConfigManager using path {}...",
            directory_path.display().to_string()
        );

        let config_exists = directory_path.exists();

        if !config_exists {
            debug!("Config directory could not be found, creating one...");

            ConfigManager::init_config(&directory_path).unwrap();

            debug!("Done creating config directory !");
        }

        let manager = ConfigManager {
            path: directory_path.clone(),
        };

        debug!(
            "Done building ConfigManager using path {} !",
            directory_path.display().to_string()
        );

        manager
    }
}

#[cfg(test)]
mod tests {

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

        let config_manager = ConfigManager::from(expected_config_file_path);

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

        let _ = ConfigManager::from(expected_config_file_path);

        // Config file is now created, try to load it once again

        let config_manager = ConfigManager::from(expected_config_file_path);

        assert_eq!(config_manager.get_path(), *expected_config_file_path);
    }

    /**
     * It should get db path
     */
    #[test]
    fn test_get_db_path() {
        let test_dir = TempDir::new().unwrap();

        let config_path = &test_dir.into_path();

        let expected_db_path = config_path.join(DB_DIR_NAME);

        let config_manager = ConfigManager::from(config_path);

        assert_eq!(config_manager.get_db_path(), *expected_db_path);
    }

    /**
     * It should get signing key
     */
    #[test]
    fn test_get_signing_key() -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = TempDir::new().unwrap();

        let expected_config_file_path = &test_dir.into_path().join("config.json");

        let config_manager = ConfigManager::from(expected_config_file_path);

        let _ = config_manager.get_signing_key()?;

        Ok(())
    }

    /**
     * It should get verifying key
     */
    #[test]
    fn test_get_verifying_key() -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = TempDir::new().unwrap();

        let expected_config_file_path = &test_dir.into_path().join("config.json");

        let config_manager = ConfigManager::from(expected_config_file_path);

        let _ = config_manager.get_verifying_key()?;

        Ok(())
    }
}
