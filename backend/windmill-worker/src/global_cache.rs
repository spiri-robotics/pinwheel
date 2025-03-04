// #[cfg(feature = "enterprise")]
// use rand::Rng;

use tokio::time::Instant;
use windmill_common::error;






pub fn extract_tar(tar: bytes::Bytes, folder: &str) -> error::Result<()> {
    use bytes::Buf;

    let start: Instant = Instant::now();
    std::fs::create_dir_all(&folder)?;

    let mut ar = tar::Archive::new(tar.reader());

    if let Err(e) = ar.unpack(folder) {
        tracing::info!("Failed to untar to {folder}. Error: {:?}", e);
        std::fs::remove_dir_all(&folder)?;
        return Err(error::Error::ExecutionErr(format!(
            "Failed to untar tar {folder}"
        )));
    }
    tracing::info!(
        "Finished extracting tar to {folder}. Took {}ms",
        start.elapsed().as_millis(),
    );
    Ok(())
}
