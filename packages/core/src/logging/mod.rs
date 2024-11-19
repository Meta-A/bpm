use std::{env, str::FromStr};

/**
 * Initializes logger
 */
pub fn init_logger(default_level: log::LevelFilter) -> log::LevelFilter {
    let custom_level = env::var("RUST_LOG");

    let level = log::LevelFilter::from_str(custom_level.unwrap_or_default().as_str())
        .unwrap_or_else(|_| default_level);

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
        env::set_var("RUST_LOG", "wwtraceww");

        let expected_level = log::LevelFilter::Trace;

        let default_level = log::LevelFilter::Debug;

        let current_log_level = init_logger(default_level);

        assert_ne!(current_log_level, expected_level);
    }
}
