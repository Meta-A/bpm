use strum_macros::{Display, EnumIter, EnumString};

/**
 * Package status
 */
#[derive(EnumIter, EnumString, PartialEq, Eq, PartialOrd, Display, Debug, Clone)]
#[repr(u8)]
pub enum PackageStatus {
    #[strum(to_string = "NA")]
    NA = 0x00,
    #[strum(to_string = "Prohibited")]
    Prohibited = 0x01,
    #[strum(to_string = "Outdated")]
    Outdated = 0x02,
    #[strum(to_string = "Fine")]
    Fine = 0x03,
    #[strum(to_string = "Recommended")]
    Recommended = 0x04,
    #[strum(to_string = "Highly recommended")]
    HighlyRecommended = 0x05,
}

impl TryFrom<u8> for PackageStatus {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PackageStatus::NA),
            1 => Ok(PackageStatus::Prohibited),
            2 => Ok(PackageStatus::Outdated),
            3 => Ok(PackageStatus::Fine),
            4 => Ok(PackageStatus::Recommended),
            5 => Ok(PackageStatus::HighlyRecommended),
            _ => Err("Invalid value for PackageStatus"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::package_status::PackageStatus;

    /**
     * It should try instantiate from u8
     */
    #[test]
    fn test_try_from() -> Result<(), Box<dyn std::error::Error>> {
        let mut expected_status = PackageStatus::NA;
        assert_eq!(PackageStatus::try_from(0 as u8)?, expected_status);

        expected_status = PackageStatus::Prohibited;
        assert_eq!(PackageStatus::try_from(1 as u8)?, expected_status);

        expected_status = PackageStatus::Outdated;
        assert_eq!(PackageStatus::try_from(2 as u8)?, expected_status);

        expected_status = PackageStatus::Fine;
        assert_eq!(PackageStatus::try_from(3 as u8)?, expected_status);

        expected_status = PackageStatus::Recommended;
        assert_eq!(PackageStatus::try_from(4 as u8)?, expected_status);

        expected_status = PackageStatus::HighlyRecommended;
        assert_eq!(PackageStatus::try_from(5 as u8)?, expected_status);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_try_from_panic() -> () {
        let wrong_status: u8 = 255;

        PackageStatus::try_from(wrong_status).unwrap();

        ()
    }
}
