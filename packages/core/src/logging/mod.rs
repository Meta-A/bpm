use std::{env, str::FromStr};

use log::debug;

/**
 * Initializes logger
 */
pub fn init_logger(default_level: log::LevelFilter) -> log::LevelFilter {
    let custom_level_result = env::var("RUST_LOG");

    let level = match custom_level_result {
        Ok(level) => log::LevelFilter::from_str(level.as_str()).unwrap_or_else(|_| default_level),
        Err(_) => {
            debug!(
                "Could not find provided log level, falling back to {}",
                default_level.as_str()
            );

            default_level
        }
    };

    env_logger::builder()
        .filter_level(level)
        .format_target(false)
        .format_timestamp(None)
        // TODO : We have to filter it because it emits warning when using tonic, find better way
        // to handle it
        .filter_module("hedera", log::LevelFilter::Error)
        .init();

    level
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
     * It should init logger with wrong log level
     */
    #[test]
    fn test_wrong_logger_initialization() {
        // I want trace but mispelled
        env::set_var("RUST_LOG", "wwwwtracewww");

        let expected_level = log::LevelFilter::Trace;

        let default_level = log::LevelFilter::Debug;

        let current_log_level = init_logger(default_level);

        assert_ne!(current_log_level, expected_level);
    }
}
