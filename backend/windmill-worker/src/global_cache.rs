
use crate::{ROOT_CACHE_DIR, ROOT_TMP_CACHE_DIR, TAR_CACHE_RATE, TMP_DIR};
use itertools::Itertools;
use rand::Rng;
use std::process::Stdio;

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

pub async fn copy_tmp_cache_to_cache() -> error::Result<()> {
    let start: Instant = Instant::now();
    execute_command(
        TMP_DIR,
        "rclone",
        vec!["sync", ROOT_TMP_CACHE_DIR, ROOT_CACHE_DIR],
    )
    .await?;
    tracing::info!(
        "Finished copying local tmp cache to local cache. Took {}ms",
        start.elapsed().as_millis(),
    );
    Ok(())
}

pub async fn copy_cache_to_tmp_cache() -> error::Result<()> {
    let start: Instant = Instant::now();
    execute_command(
        TMP_DIR,
        "rclone",
        vec!["sync", ROOT_CACHE_DIR, ROOT_TMP_CACHE_DIR],
    )
    .await?;
    tracing::info!(
        "Finished copying local cache to local tmp cache. Took {}ms",
        start.elapsed().as_millis()
    );
    Ok(())
}

pub async fn execute_command(dir: &str, command: &str, args: Vec<&str>) -> error::Result<()> {
    match Command::new(command)
        .current_dir(dir)
        .args(args.clone())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
    {
        Ok(mut h) => {
            if !h.wait().await.unwrap().success() {
                return Err(error::Error::ExecutionErr(format!(
                    "Failed to apply {command} with args: {}",
                    args.iter().join(" ")
                )));
            }
        }
        Err(e) => {
            return Err(error::Error::ExecutionErr(format!(
                "Failed to apply {command} with args: {}. Error: {e:?}",
                args.iter().join(" ")
            )));
        }
    }
    Ok(())
}
