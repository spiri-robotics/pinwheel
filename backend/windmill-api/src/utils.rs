/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2022
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

use axum::{body::Body, response::Response};
use serde::{Deserialize, Deserializer};
use windmill_common::{
    error::{self},
    DB,
};

pub use windmill_api_auth::{check_scopes, require_devops_role, require_super_admin};



pub use windmill_common::usernames::generate_instance_wide_unique_username;
pub use windmill_common::utils::WithStarredInfoQuery;

pub async fn generate_instance_username_for_all_users(db: &DB) -> error::Result<()> {
    let mut tx = db.begin().await?;
    // get users that have a no instance username and either 1 or 0 workspace usernames
    let users = sqlx::query!(r#"SELECT p.email as "email!", u.username as "username?" FROM password p LEFT JOIN usr u ON p.email = u.email WHERE p.username IS NULL AND (SELECT COUNT(DISTINCT username) FROM usr WHERE email = p.email) <= 1"#)
        .fetch_all(&mut *tx)
        .await?;

    for user in users {
        let username = if let Some(username) = user.username {
            // if has workspace username, check that username is unique
            let username_conflict = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM usr WHERE username = $1 and email != $2 UNION SELECT 1 FROM password WHERE username = $1 UNION SELECT 1 FROM pending_user WHERE username = $1)",
                &username,
                &user.email
            ).fetch_one(&mut *tx).await?.unwrap_or(false);

            if !username_conflict {
                username
            } else {
                generate_instance_wide_unique_username(&mut tx, &user.email).await?
            }
        } else {
            generate_instance_wide_unique_username(&mut tx, &user.email).await?
        };

        sqlx::query!(
            "UPDATE password SET username = $1 WHERE email = $2",
            &username,
            &user.email
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub fn content_plain(body: Body) -> Response {
    use axum::http::header;
    Response::builder()
        .header(header::CONTENT_TYPE, "text/plain")
        .body(body)
        .unwrap()
}

#[allow(unused)]
pub fn non_empty_str<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Option::deserialize(deserializer)?;
    Ok(o.filter(|s| !s.trim().is_empty()))
}






#[cfg(feature = "http_trigger")]
pub use windmill_common::utils::ExpiringCacheEntry;

lazy_static::lazy_static! {
    static ref DUCKLAKE_INSTANCE_PG_PASSWORD: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);
}
