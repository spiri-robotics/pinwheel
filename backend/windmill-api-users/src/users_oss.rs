
use crate::users::ImpersonateServiceAccountRequest;
use http::StatusCode;
use tower_cookies::Cookies;
use windmill_api_auth::ApiAuthed;
use windmill_common::DB;

pub async fn impersonate_service_account(
    _db: DB,
    _authed: ApiAuthed,
    _cookies: Cookies,
    _current_token: String,
    _w_id: String,
    _req: ImpersonateServiceAccountRequest,
) -> windmill_common::error::Result<(StatusCode, String)> {
    Err(windmill_common::error::Error::BadRequest(
        "Service accounts require Windmill Enterprise Edition".to_string(),
    ))
}
