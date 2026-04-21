use tokio::time::Instant;
use windmill_common::error;



pub const TARGET: &str = const_format::concatcp!(std::env::consts::OS, "_", std::env::consts::ARCH);



pub fn extract_tar(tar: bytes::Bytes, folder: &str) -> error::Result<()> {
    use bytes::Buf;

    let start: Instant = Instant::now();
    std::fs::create_dir_all(&folder)?;

    let mut ar = tar::Archive::new(tar.reader());

    if let Err(e) = ar.unpack(folder) {
        tracing::info!("Failed to untar to {folder}. Error: {:?}", e);
        std::fs::remove_dir_all(&folder)?;
        return Err(error::Error::ExecutionErr(format!(
            "Failed to untar tar {folder}. Error: {:?}",
            e
        )));
    }
    tracing::info!(
        "Finished extracting tar to {folder}. Took {}ms",
        start.elapsed().as_millis(),
    );
    Ok(())
}

/// Two-tier cache load: check local disk first, then fall back to instance object store.
/// Returns `(hit, log_message)`.
pub async fn load_cache(bin_path: &str, _remote_path: &str, is_dir: bool) -> (bool, String) {
    if tokio::fs::metadata(&bin_path).await.is_ok() {
        (true, format!("loaded from local cache: {}\n", bin_path))
    } else {
        let _ = is_dir;
        (false, "".to_string())
    }
}

/// Check whether a binary/bundle exists in local cache or instance object store.
pub async fn exists_in_cache(bin_path: &str, _remote_path: &str) -> bool {
    if tokio::fs::metadata(&bin_path).await.is_ok() {
        return true;
    } else {
        return false;
    }
}

/// Two-tier cache write: upload to instance object store, then copy to local disk.
pub async fn save_cache(
    local_cache_path: &str,
    _remote_cache_path: &str,
    origin: &str,
    is_dir: bool,
) -> windmill_common::error::Result<String> {
    use std::path::PathBuf;

    let mut _cached_to_s3 = false;

    if true {
        if is_dir {
            windmill_common::worker::copy_dir_recursively(
                &PathBuf::from(origin),
                &PathBuf::from(local_cache_path),
            )?;
        } else {
            std::fs::copy(origin, local_cache_path)?;
        }
        Ok(format!(
            "\nwrote cached binary: {} (backed by EE distributed object store: {_cached_to_s3})\n",
            local_cache_path
        ))
    } else if _cached_to_s3 {
        Ok(format!(
            "wrote cached binary to object store {}\n",
            local_cache_path
        ))
    } else {
        Ok("".to_string())
    }
}
