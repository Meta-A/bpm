use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use log::{debug, trace};
use walkdir::WalkDir;

/**
 * Returns whether file is an executable or not
 */
fn is_executable(metadata: &fs::Metadata) -> bool {
    metadata.is_file() && metadata.mode() & 0o111 != 0
}

/**
 * Find all executables in given directory and its subdirectories
 */
pub fn find_executables(directory: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    debug!(
        "Searching executables in {}...",
        directory.to_path_buf().display()
    );
    let mut executables_paths = vec![];
    for entry in WalkDir::new(directory) {
        let entry = entry?;

        let entry_metadata = entry.metadata()?;

        let entry_path = entry.into_path().clone();

        if is_executable(&entry_metadata) {
            trace!("Found executable at {}", entry_path.display());
            executables_paths.push(entry_path);
        }
    }

    debug!("Done finding executables !");
    Ok(executables_paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs::File;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    /**
     * It should return false if not executable
     */
    #[test]
    fn test_executable_on_non_executable_file() -> Result<(), Box<dyn std::error::Error>> {
        let expected_executable_state = false;

        let test_dir = TempDir::new().unwrap();

        let file_path = test_dir.path().join("non_executable");

        // Create the file
        let file = File::create(&file_path)?;

        let metadata = fs::metadata(file_path)?;
        let executable = is_executable(&metadata);

        assert_eq!(executable, expected_executable_state);

        Ok(())
    }

    /**
     * It should return true if executable
     */
    #[test]
    fn test_executable_on_executable_file() -> Result<(), Box<dyn std::error::Error>> {
        let expected_executable_state = true;

        let test_dir = TempDir::new().unwrap();

        let file_path = test_dir.path().join("executable");

        // Create the file
        let file = File::create(&file_path)?;

        // Set permissions to make the file executable
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&file_path, permissions)?;

        let metadata = fs::metadata(file_path)?;
        let executable = is_executable(&metadata);

        assert_eq!(executable, expected_executable_state);

        Ok(())
    }

    /**
     * It should return only executable files
     */
    #[test]
    fn test_find_executable_files() -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = TempDir::new().unwrap();

        let file_path_one = test_dir.path().join("executable_1");
        let file_path_two = test_dir.path().join("executable_2");
        let file_path_three = test_dir.path().join("non_executable");

        // Create the files
        let file_one = File::create(&file_path_one)?;
        let file_two = File::create(&file_path_two)?;
        File::create(&file_path_three)?;

        let expected_executable_files = vec![file_path_one.clone(), file_path_two.clone()];

        // Set permissions to make the files executable
        let mut permissions = file_one.metadata()?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&file_path_one, permissions)?;

        permissions = file_two.metadata()?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&file_path_two, permissions)?;

        let executable_files = find_executables(test_dir.path())?;

        assert_eq!(expected_executable_files, executable_files);

        Ok(())
    }
}
