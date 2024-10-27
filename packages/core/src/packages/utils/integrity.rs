use crate::{
    packages::package::PackageBinariesIntegrityMap, utils::fs::unix::executables::find_executables,
};
use futures_util::stream::FuturesUnordered;
use log::{debug, trace};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, path::PathBuf};

/**
 * Compute hash for single file
 */
async fn compute_package_file_hash(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut hasher = Sha256::new(); // TODO : pass hasher through params
    let path = path.as_path().to_str().unwrap();
    let content = tokio::fs::read(path).await.unwrap();

    hasher.update(content);

    let result = hasher.finalize();

    let encoded_hash = hex::encode(result);

    Ok(encoded_hash)
}

/**
 * Compute hash for each binary found and associate it with its relative path
 */
pub async fn compute_package_binaries_hashes(
    directory: PathBuf,
) -> Result<PackageBinariesIntegrityMap, Box<dyn std::error::Error>> {
    debug!(
        "Computing package binaries hashes in directory {}...",
        directory.display()
    );

    let executables = find_executables(&directory.as_path())?;

    // For each path, map task to read binary and calculate its hash
    let executable_path_to_hash_futures = executables
        .iter()
        .cloned()
        .map(|executable_path| {
            let base_path = directory.clone();

            tokio::spawn(async move {
                trace!(
                    "Computing hash for binary located at {}",
                    executable_path.display()
                );

                let hash = compute_package_file_hash(&executable_path).await.unwrap();

                let relative_path = PathBuf::from(executable_path.strip_prefix(base_path).unwrap());

                trace!(
                    "Done computing hash for binary located at {}, result : {} !",
                    executable_path.as_path().display(),
                    hash
                );

                (relative_path, hash)
            })
        })
        .collect::<FuturesUnordered<_>>();

    let futures_results = futures_util::future::join_all(executable_path_to_hash_futures).await;

    let mut relative_executable_path_to_hash: HashMap<PathBuf, String> = HashMap::new();

    // Insert path to hash in map
    for future_result in futures_results {
        let (binary_relative_path, hash) = future_result?;
        relative_executable_path_to_hash.insert(binary_relative_path, hash);
    }

    debug!(
        "Done computing package binaries hashes in directory {} !",
        directory.display()
    );

    Ok(relative_executable_path_to_hash)
}

/**
 * Compute hash sum of package content
 */
pub async fn compute_package_archive_hash(
    archive_path: PathBuf,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!(
        "Computing package archive hash located at {}...",
        archive_path.display()
    );

    let archive_path_hash = compute_package_file_hash(&archive_path).await?;

    debug!(
        "Done computing package archive hash located at {}...",
        archive_path.display()
    );

    Ok(archive_path_hash)
}
