
use crate::{ROOT_CACHE_DIR, ROOT_TMP_CACHE_DIR};
use windmill_common::error;






// async fn check_if_bucket_syncable(bucket: &str) -> bool {
//     match Command::new("rclone")
//         .arg("lsf")
//         .arg(format!(":s3,env_auth=true:{bucket}/NOSYNC"))

//         .arg("-vv")
//         .arg("--fast-list")
//         .stdin(Stdio::null())
//         .stdout(Stdio::null())
//         .output()
//         .await;
//     return true;
// }

pub async fn move_tmp_cache_to_cache() -> error::Result<()> {
    tokio::fs::remove_dir_all(ROOT_CACHE_DIR).await?;
    tokio::fs::rename(ROOT_TMP_CACHE_DIR, ROOT_CACHE_DIR).await?;
    tracing::info!("Finished moving tmp cache to cache");
    Ok(())
}
