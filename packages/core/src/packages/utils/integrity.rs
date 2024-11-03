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

    // It should compute hash
    //#[tokio::test]
    //async fn test_compute_file_hash() {
    //    let test_dir = TempDir::new().unwrap();
    //
    //    let file_path = test_dir.path().join("file_to_hash");
    //    let mut file = File::create(&file_path).unwrap();
    //
    //    let content = "Hello, world";
    //
    //    // Write the string to the file
    //    file.write_all(content.as_bytes()).unwrap();
    //
    //    let mut hasher = Sha256::new();
    //    hasher.update(content.as_bytes());
    //    let expected_hash = hex::encode(hasher.finalize());
    //
    //    let hash = compute_package_file_hash(&file_path).await.unwrap();
    //    assert_eq!(expected_hash, hash);
    //}
}
