enum Status {
    NA = 0,
    PROHIBITED = 1,
    OUTDATED = 2,
    FINE = 3,
    RECOMMENDED = 4,
    HIGHLY_RECOMMENDED = 5,
}

struct PackageStatus {
    status: Status,
    color: String,
}

#[derive(Default)]
pub struct PackageStatusBuilder {
    status: Option<Status>,
    color: Option<String>,
}

impl PackageStatusBuilder {
    pub fn new() -> Self {
        Self {
            status: None,
            color: None,
        }
    }

    pub fn set_status(&mut self, status: Status) -> Self {
        self.status = Some(status);

        self
    }

    pub fn set_color(&mut self, color: String) -> Self {
        self.color = Some(color);

        self
    }

    pub fn build(&self) -> PackageStatus {
        PackageStatus {
            status: self.status,
        }
    }
}
