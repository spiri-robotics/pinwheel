
use axum::Router;
use uuid::Uuid;
use windmill_types::s3::StorageResourceType;

#[cfg(feature = "parquet")]
use crate::db::{ApiAuthed, OptJobAuthed, DB};
#[cfg(feature = "parquet")]
use std::sync::Arc;
#[cfg(feature = "parquet")]
use windmill_common::db::UserDB;
use windmill_common::error;
#[cfg(feature = "parquet")]
use windmill_object_store::object_store_reexports::{ObjectStore, PutMultipartOpts, PutResult};
use windmill_object_store::ObjectStoreResource;

#[cfg(feature = "parquet")]
use bytes::Bytes;
#[cfg(feature = "parquet")]
use futures::Stream;

#[cfg(feature = "parquet")]
use axum::response::Response;
#[cfg(feature = "parquet")]
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DownloadFileQuery {
    #[allow(dead_code)]
    pub file_key: String,
    #[allow(dead_code)]
    pub storage: Option<String>,
    #[allow(dead_code)]
    pub s3_resource_path: Option<String>,
}

pub fn workspaced_service() -> Router {
    Router::new()
}

#[cfg(feature = "parquet")]
pub async fn get_workspace_s3_resource<'c>(
    _authed: &ApiAuthed,
    _db: &DB,
    _user_db: Option<UserDB>,
    _w_id: &str,
    _storage: Option<String>,
) -> windmill_common::error::Result<(Option<bool>, Option<ObjectStoreResource>)> {
    // implementation is not open source
    Ok((None, None))
}

pub fn get_random_file_name(_file_extension: Option<String>) -> String {
    unimplemented!("Not implemented in Windmill's Open Source repository")
}

use windmill_common::db::DbWithOptAuthed;

pub async fn get_s3_resource<'c>(
    _db_with_opt_authed: &DbWithOptAuthed<'c, ApiAuthed>,
    _w_id: &str,
    _resource_path: &str,
    _resource_type: Option<StorageResourceType>,
    _job_id: Option<Uuid>,
) -> error::Result<ObjectStoreResource> {
    Err(error::Error::internal_err(
        "Not implemented in Windmill's Open Source repository".to_string(),
    ))
}

#[cfg(feature = "parquet")]
pub async fn upload_file_from_req(
    _s3_client: Arc<dyn ObjectStore>,
    _file_key: &str,
    _req: axum::extract::Request,
    _options: PutMultipartOpts,
    _max_size: Option<usize>,
) -> error::Result<(PutResult, usize)> {
    Err(error::Error::internal_err(
        "Not implemented in Windmill's Open Source repository".to_string(),
    ))
}

#[cfg(feature = "parquet")]
pub async fn upload_file_internal(
    _s3_client: Arc<dyn ObjectStore>,
    _file_key: &str,
    _stream: impl Stream<Item = Result<Bytes, std::io::Error>> + Unpin,
    _options: PutMultipartOpts,
    _max_size: Option<usize>,
) -> error::Result<(PutResult, usize)> {
    Err(error::Error::internal_err(
        "Not implemented in Windmill's Open Source repository".to_string(),
    ))
}

// These stubs stand in for the CE quota helpers in a pure-OSS build; their only
// callers (apps.rs / args.rs uploads) are `not(enterprise)`, so gate them the
// same way — an enterprise-without-private build compiles neither.
#[cfg(feature = "parquet")]
pub async fn ce_storage_quota_remaining(
    _db: &DB,
    _w_id: &str,
    _exclude_upload_id: Option<&str>,
) -> error::Result<i64> {
    Ok(i64::MAX)
}

#[cfg(feature = "parquet")]
pub fn reject_reserved_volume_key(_file_key: &str) -> error::Result<()> {
    Ok(())
}

#[cfg(feature = "parquet")]
pub struct CeUploadBudget {
    pub max_size: usize,
    pub existing_size: i64,
}

#[cfg(feature = "parquet")]
pub async fn ce_upload_budget(
    _db: &DB,
    _w_id: &str,
    _s3_client: &Arc<dyn ObjectStore>,
    _file_key: &str,
    _content_length: Option<i64>,
) -> error::Result<CeUploadBudget> {
    Ok(CeUploadBudget { max_size: usize::MAX, existing_size: 0 })
}

#[cfg(feature = "parquet")]
pub async fn bump_storage_usage(_db: &DB, _w_id: &str, _storage: &str, _delta: i64) {}

#[cfg(feature = "parquet")]
pub fn spawn_storage_usage_recount_floored(_db: &DB, _w_id: &str) {}

#[cfg(feature = "parquet")]
pub async fn download_s3_file_internal(
    _authed: OptJobAuthed,
    _db: &DB,
    _user_db: Option<UserDB>,
    _w_id: &str,
    _query: DownloadFileQuery,
) -> error::Result<Response> {
    Err(error::Error::internal_err(
        "Not implemented in Windmill's Open Source repository".to_string(),
    ))
}

#[allow(dead_code)]
pub async fn read_object_streamable(
    _s3_client: Arc<dyn ObjectStore>,
    _file_key: &str,
) -> error::Result<Response> {
    Err(error::Error::internal_err(
        "Not implemented in Windmill's Open Source repository".to_string(),
    ))
}

#[allow(dead_code)]
pub async fn delete_s3_file_internal(
    _authed: OptJobAuthed,
    _db: &DB,
    _token: &str,
    _w_id: &str,
    _query: DeleteS3FileQuery,
) -> error::Result<()> {
    Err(error::Error::internal_err(
        "Not implemented in Windmill's Open Source repository".to_string(),
    ))
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct DeleteS3FileQuery {
    pub file_key: String,
    pub storage: Option<String>,
}

pub async fn get_workspace_s3_resource_and_check_paths<'c>(
    _db_with_opt_authed: &DbWithOptAuthed<'c, ApiAuthed>,
    _authed_api: Option<&ApiAuthed>,
    _w_id: &str,
    _storage: Option<String>,
    _paths: &[(&str, windmill_types::s3::S3Permission)],
    _job_id: Option<uuid::Uuid>,
) -> windmill_common::error::Result<(
    Option<bool>,
    Option<windmill_types::s3::ObjectStoreResource>,
)> {
    Err(windmill_common::error::Error::internal_err(
        "Not implemented in Windmill's Open Source repository".to_string(),
    ))
}
