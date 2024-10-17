/**
 * Initializes logger
 */
pub fn init_logger(level: log::LevelFilter) -> log::LevelFilter {
    env_logger::builder()
        .filter_level(level)
        .format_target(false)
        .format_timestamp(None)
        .init();

    log::max_level()
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
     * It should init logger with right log level
     */
    #[test]
    fn test_logger_initialization() {
        let expected_level = log::LevelFilter::Debug;
        let current_log_level = init_logger(expected_level);

        assert_eq!(current_log_level, expected_level);
    }
}
