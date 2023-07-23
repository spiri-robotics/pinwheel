/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2023
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

use axum::{
    extract::{Path, Query},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use bytes::{BufMut, BytesMut};
use hyper::{header, http::HeaderValue, Request, StatusCode};
use mime_guess::mime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sql_builder::SqlBuilder;
use windmill_common::{
    error::{Error, Result},
    utils::not_found_if_none,
};

use crate::db::DB;

lazy_static::lazy_static! {
    static ref SCIM_TOKEN: Option<String> = std::env::var("SCIM_TOKEN")
        .ok();
}


pub fn global_service() -> Router {
    Router::new().route("/Users", get(get_users))
}

#[derive(Debug, Clone, Copy, Default)]
pub struct JsonScim<T>(pub T);

pub async fn has_scim_token<B>(request: Request<B>, next: Next<B>) -> Response {
    let header = request.headers().get("Authorization");
    if let Some(header) = header {
        if let Ok(header) = header.to_str() {
            if header.starts_with("Bearer ") {
                let token = header.trim_start_matches("Bearer ");
                if let Some(scim_token) = SCIM_TOKEN.as_ref() {
                    if token == scim_token {
                        return next.run(request).await;
                    }
                }
            }
        }
    }
    return (
        StatusCode::UNAUTHORIZED,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
        )],
        "Unauthorized",
    )
        .into_response();
}

pub type JsonScimResult<T> = std::result::Result<JsonScim<T>, Error>;

impl<T> IntoResponse for JsonScim<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_json::to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/scim+json"),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}

#[derive(Serialize, Debug)]
struct User {
    id: String,
    userName: String,
    active: bool,
}
pub fn resource_response<S>(schema: &str, resources: Vec<S>) -> JsonScim<serde_json::Value>
where
    S: Serialize,
{
    return JsonScim(json!({
        "schemas": [schema],
        "totalResults": resources.len(),
        "Resources": resources,
        "startIndex": 1,
        "itemsPerPage": 100,
    }));
}

#[derive(Deserialize)]
pub struct ScimQuery {
    startIndex: Option<u32>,
    count: Option<u32>,
    filter: Option<String>,
}

pub async fn get_users(
    Extension(db): Extension<DB>,
    Query(query): Query<ScimQuery>,
) -> Result<JsonScim<serde_json::Value>> {
    let mut sqlb = SqlBuilder::select_from("usr")
        .fields(&["email"])
        .limit(query.count.unwrap_or(100000))
        .offset(query.startIndex.map(|x| x - 1).unwrap_or(0))
        .clone();

    tracing::info!("SCIM filter: {:?}", query.filter);

    if let Some(filter) = query.filter {
        let filter = filter
            .replace("userName", "email")
            .replace("eq", "=")
            .replace("\"", "'");
        sqlb.and_where(&filter);
    }

    let sql = sqlb.sql().map_err(|e| Error::InternalErr(e.to_string()))?;
    let users = sqlx::query_scalar(&sql)
        .fetch_all(&db)
        .await?
        .into_iter()
        .map(|x: String| User { id: x.clone(), userName: x, active: true })
        .collect();
    tracing::info!("SCIM users: {:?}", users);
    Ok(resource_response(
        "urn:ietf:params:scim:api:messages:2.0:ListResponse",
        users,
    ))
}

#[derive(Deserialize, Debug)]
pub struct CreateUser {
    userName: String,
}
// #[cfg(feature = "enterprise")]
pub async fn create_user(
    Extension(db): Extension<DB>,
    Json(body): Json<CreateUser>,
) -> Result<JsonScim<serde_json::Value>> {
    tracing::info!("SCIM creating user: {:?}", body);
    sqlx::query!(
        "INSERT INTO password (email, login_type, verified) VALUES ($1, 'saml', true) ON CONFLICT DO NOTHING",
        body.userName,
    ).execute(&db).await?;
    Ok(JsonScim(json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
        "id": body.userName,
        "userName": body.userName,
        "active": true
    })))
}


// {
//     "schemas": [],
//     "id": "abf4dd94-a4c0-4f67-89c9-76b03340cb9b",
//     "displayName": "Test SCIMv2",
//     "members": [],
//     "meta": {
//         "resourceType": "Group"
//     }
// }



// {
//     "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Group"],
//     "displayName": "Test SCIMv2",
//     "members": []
// }




#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Operation {
    pub op: String,
    pub path: String,
    pub value: Option<serde_json::Value>,
}



pub async fn delete_group(Extension(db): Extension<DB>, Path(id): Path<String>) -> Result<()> {
    tracing::info!("SCIM delete group: {:?}", id);
    sqlx::query!("DELETE FROM email_to_igroup WHERE igroup = $1", id)
        .execute(&db)
        .await?;
    sqlx::query!("DELETE FROM instance_group WHERE name = $1", id)
        .execute(&db)
        .await?;
    Ok(())
}

fn convert_name(name: &str) -> String {
    name.replace(" ", "_").to_lowercase()
}
