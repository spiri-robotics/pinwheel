



// #[cfg(feature = "enterprise")]
// const TAR_CACHE_FILENAME: &str = "denogocache.tar";



// #[cfg(feature = "enterprise")]
// pub async fn cache_global(bucket: &str, _tx: Sender<()>) -> error::Result<()> {
//     // copy_cache_from_bucket(bucket, tx).await?;
//     // copy_cache_to_bucket(bucket).await?;

//     // this is to prevent excessive tar upload. 1/100*15min = each worker sync its tar once per day on average
//     if rand::thread_rng().gen_range(0..*TAR_CACHE_RATE) == 0 {
//         copy_cache_to_bucket_as_tar(bucket).await;
//     }
//     Ok(())
// }

// #[cfg(feature = "enterprise")]
// pub async fn copy_cache_from_bucket(bucket: &str, tx: Sender<()>) -> error::Result<()> {
//     tracing::info!("Copying cache from bucket in the background {bucket}");
//     let bucket = bucket.to_string();

//     let start = Instant::now();

//     if let Err(e) = execute_command(
//         ROOT_TMP_CACHE_DIR,
//         "rclone",
//         vec![
//             "copy",
//             &format!(":s3,env_auth=true:{bucket}"),
//             &ROOT_TMP_CACHE_DIR,
//             // "-l",
//             "--size-only",
//             "--fast-list",
//             "--filter",
//             "+ deno/npm/**",
//             "--filter",
//             "+ deno/deps/**",
//             // "--filter",
//             // "+ bun/**",
//             "--filter",
//             "+ go/**",
//             "--filter",
//             "+ tar/**",
//             "--filter",
//             "- *",
//         ],
//     )
//     .await
//     {
//         tracing::info!("Failed to copy cache from bucket. Error: {:?}", e);
//         return Err(e);
//     }

//     tracing::info!(
//         "Finished copying cache from bucket {bucket}, took {:?}s",
//         start.elapsed().as_secs()
//     );

//     tx.send(()).await.expect("can send copy cache signal");

//     Ok(())
// }

// #[cfg(feature = "enterprise")]
// pub async fn copy_cache_to_bucket(bucket: &str) -> error::Result<()> {
//     tracing::info!("Copying cache to bucket {bucket}");
//     let start = Instant::now();

//     if let Err(e) = execute_command(
//         ROOT_TMP_CACHE_DIR,
//         "rclone",
//         vec![
//             "copy",
//             &ROOT_TMP_CACHE_DIR,
//             &format!(":s3,env_auth=true:{bucket}"),
//             // "-l",
//             "--size-only",
//             "--fast-list",
//             "--filter",
//             "+ deno/npm/**",
//             "--filter",
//             "+ deno/deps/**",
//             // "--filter",
//             // "+ bun/**",
//             "--filter",
//             "+ go/**",
//             "--filter",
//             "- *",
//         ],
//     )
//     .await
//     {
//         tracing::info!("Failed to copy cache to bucket. Error: {:?}", e);
//         return Err(e);
//     }
//     tracing::info!(
//         "Finished copying cache to bucket {bucket}, took: {:?}s",
//         start.elapsed().as_secs()
//     );
//     Ok(())
// }


// #[cfg(feature = "enterprise")]
// pub async fn copy_cache_to_bucket_as_tar(bucket: &str) {
//     tracing::info!("Copying cache to bucket {bucket} as tar");
//     let start = Instant::now();

//     if let Err(e) = execute_command(
//         ROOT_TMP_CACHE_DIR,
//         "tar",
//         vec![
//             "-c",
//             "-f",
//             &format!("{ROOT_TMP_CACHE_DIR}{TAR_CACHE_FILENAME}"),
//             "go",
//             "deno/npm",
//             "deno/deps", // "bun",
//         ],
//     )
//     .await
//     {
//         tracing::info!("Failed to tar cache. Error: {:?}", e);
//         return;
//     }

//     let tar_metadata =
//         tokio::fs::metadata(format!("{ROOT_TMP_CACHE_DIR}{TAR_CACHE_FILENAME}")).await;
//     if tar_metadata.is_err() || tar_metadata.as_ref().unwrap().len() == 0 {
//         tracing::info!("Failed to tar cache");
//         return;
//     }

//     if let Err(e) = execute_command(
//         ROOT_TMP_CACHE_DIR,
//         "rclone",
//         vec![
//             "copyto",
//             &format!("{ROOT_TMP_CACHE_DIR}{TAR_CACHE_FILENAME}"),
//             &format!(":s3,env_auth=true:{bucket}/{TAR_CACHE_FILENAME}"),
//             "-v",
//             "--size-only",
//             "--fast-list",
//             "--s3-no-check-bucket",
//         ],
//     )
//     .await
//     {
//         tracing::info!("Failed to copy tar to bucket. Error: {:?}", e);
//         return;
//     }

//     if let Err(e) =
//         tokio::fs::remove_file(format!("{ROOT_TMP_CACHE_DIR}{TAR_CACHE_FILENAME}")).await
//     {
//         tracing::info!("Failed to remove tar cache. Error: {:?}", e);
//     };

//     tracing::info!(
//         "Finished copying cache to bucket {bucket} as tar, took: {:?}s. Size of new tar: {}",
//         start.elapsed().as_secs(),
//         tar_metadata.unwrap().len()
//     );
// }

// #[cfg(feature = "enterprise")]
// pub async fn copy_denogo_cache_from_bucket_as_tar(bucket: &str) {
//     tracing::info!("Copying deno,go,bun cache from bucket {bucket} as tar");

//     let mut start: Instant = Instant::now();

//     if let Err(e) = execute_command(
//         ROOT_TMP_CACHE_DIR,
//         "rclone",
//         vec![
//             "copyto",
//             &format!(":s3,env_auth=true:{bucket}/{TAR_CACHE_FILENAME}"),
//             &format!("{ROOT_TMP_CACHE_DIR}{TAR_CACHE_FILENAME}"),
//             "-v",
//             "--size-only",
//             "--fast-list",
//         ],
//     )
//     .await
//     {
//         tracing::info!("Failed copying deno,go,bun tar from cache. Error: {:?}", e);
//         return;
//     }

//     tracing::info!(
//         "Finished copying denogobun tar for from bucket as tar. took {}s",
//         start.elapsed().as_secs()
//     );

//     start = Instant::now();

//     if let Err(e) = execute_command(
//         ROOT_CACHE_DIR,
//         "tar",
//         vec![
//             "-xpvf",
//             &format!("{ROOT_TMP_CACHE_DIR}{TAR_CACHE_FILENAME}"),
//         ],
//     )
//     .await
//     {
//         tracing::info!("Failed to untar denogobun tar to cache. Error: {:?}", e);
//         return;
//     }

//     tracing::info!(
//         "Finished untaring denogobun tar to cache. took: {}s",
//         start.elapsed().as_secs()
//     );

//     start = Instant::now();

//     if let Err(e) = execute_command(
//         ROOT_TMP_CACHE_DIR,
//         "tar",
//         vec![
//             "-xpvf",
//             &format!("{ROOT_TMP_CACHE_DIR}{TAR_CACHE_FILENAME}"),
//         ],
//     )
//     .await
//     {
//         tracing::info!("Failed to untar denogobun tar to tmpcache. Error: {:?}", e);
//         return;
//     }

//     tracing::info!(
//         "Finished untaring denogobun tar to /tmpcache. took: {}s",
//         start.elapsed().as_secs()
//     );

//     if let Err(e) =
//         tokio::fs::remove_file(format!("{ROOT_TMP_CACHE_DIR}{TAR_CACHE_FILENAME}")).await
//     {
//         tracing::info!("Failed to remove denogobuntar cache. Error: {:?}", e);
//     };
// }


// #[cfg(feature = "enterprise")]
// pub async fn copy_tmp_cache_to_cache() -> error::Result<()> {
//     let start: Instant = Instant::now();
//     execute_command(
//         TMP_DIR,
//         "rclone",
//         vec![
//             "sync",
//             ROOT_TMP_CACHE_DIR,
//             ROOT_CACHE_DIR,
//             // "-l",
//             "--filter",
//             "+ deno/npm/**",
//             "--filter",
//             "+ deno/deps/**",
//             // "--filter",
//             // "+ bun/**",
//             "--filter",
//             "+ go/**",
//             "--filter",
//             "- *",
//         ],
//     )
//     .await?;

//     tracing::info!(
//         "Finished copying local tmp cache to local cache. Took {}ms",
//         start.elapsed().as_millis(),
//     );

//     if let Err(e) = untar_all_piptars().await {
//         tracing::info!("Failed to untar piptars. Error: {:?}", e);
//     }

//     Ok(())
// }



// #[cfg(feature = "enterprise")]
// pub async fn copy_cache_to_tmp_cache() -> error::Result<()> {
//     let start: Instant = Instant::now();
//     execute_command(
//         TMP_DIR,
//         "rclone",
//         vec![
//             "sync",
//             ROOT_CACHE_DIR,
//             ROOT_TMP_CACHE_DIR,
//             // "-l",
//             "--filter",
//             "+ deno/npm/**",
//             "--filter",
//             "+ deno/deps/**",
//             // "--filter",
//             // "+ bun/**",
//             "--filter",
//             "+ go/**",
//             "--filter",
//             "- *",
//         ],
//     )
//     .await?;
//     tracing::info!(
//         "Finished copying local cache to local tmp cache. Took {}ms",
//         start.elapsed().as_millis()
//     );
//     Ok(())
// }

