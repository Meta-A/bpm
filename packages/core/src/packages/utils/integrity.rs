use sha2::{Digest, Sha256};
use std::path::PathBuf;

/**
 * Compute hash for single file
 */
pub async fn compute_package_file_hash(
    path: &PathBuf,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new(); // TODO : pass hasher through params

    let path = path.as_path().to_str().unwrap();

    let content = tokio::fs::read(path).await.unwrap();

    hasher.update(content);

    let result = hasher.finalize();

    let hash = Vec::from(result.as_slice());
    let algorithm = "SHA256".to_string();

    Ok((hash, algorithm))
}

#[cfg(test)]
mod tests {

    use std::{fs::File, io::Write};

    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_compute_package_file_hash() -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = TempDir::new().unwrap();

        let test_file_path = test_dir.path().join("test.txt");

        let hashed_content = "foo";

        let mut file = File::create_new(&test_file_path).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(hashed_content);
        let expected_hash = hasher.finalize().to_vec();

        file.write(hashed_content.as_bytes())?;

        let (hash, _) = compute_package_file_hash(&test_file_path).await?;

        assert_eq!(hash, expected_hash);

        Ok(())
    }
}
