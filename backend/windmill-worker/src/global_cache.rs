








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



