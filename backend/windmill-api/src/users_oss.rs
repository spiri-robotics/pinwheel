
use std::sync::Arc;

use crate::db::ApiAuthed;

use crate::users::{EditPassword, NewUser};

use crate::{db::DB, webhook_util::WebhookShared};

use argon2::{Argon2, PasswordHasher};

use axum::{extract::Extension, Json};

use http::StatusCode;

use serde::Deserialize;

use windmill_api_auth::require_super_admin;
use windmill_audit::audit_oss::audit_log;
use windmill_audit::ActionKind;
use windmill_common::email_oss::send_email_if_possible;
use windmill_common::error::{Error, Result};
use windmill_common::global_settings::AUTOMATE_USERNAME_CREATION_SETTING;
use windmill_common::usernames::generate_instance_wide_unique_username;
use windmill_common::users::{EMAIL_COLUMN_MAX_LEN, VALID_EMAIL};

const PASSWORD_LOGIN_TYPE: &str = "password";
/// Accounts backed by an LDAP directory: their password lives there, not here.
const LDAP_LOGIN_TYPE: &str = "ldap";

pub async fn create_user(
    authed: ApiAuthed,
    db: DB,
    _webhook: WebhookShared,
    argon2: Arc<Argon2<'_>>,
    nu: NewUser,
) -> Result<(StatusCode, String)> {
    require_super_admin(&db, &authed).await?;

    let email = nu.email.trim().to_lowercase();
    if !VALID_EMAIL.is_match(&email) || email.len() > EMAIL_COLUMN_MAX_LEN {
        return Err(Error::BadRequest(format!("invalid email: {email}")));
    }

    let login_type = nu
        .login_type
        .unwrap_or_else(|| PASSWORD_LOGIN_TYPE.to_string());
    if login_type == LDAP_LOGIN_TYPE {
        return Err(Error::BadRequest(
            "LDAP accounts are created on their first login".to_string(),
        ));
    }
    let password_hash = if login_type == PASSWORD_LOGIN_TYPE {
        let password = nu
            .password
            .filter(|p| !p.is_empty())
            .ok_or_else(|| Error::BadRequest("password is required".to_string()))?;
        Some(hash_password(argon2, password)?)
    } else {
        None
    };

    let mut tx = db.begin().await?;

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM password WHERE email = $1)")
        .bind(&email)
        .fetch_one(&mut *tx)
        .await?;
    if exists {
        return Err(Error::BadRequest(format!("user {email} already exists")));
    }

    let automate_username_creation = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT value FROM global_settings WHERE name = $1",
    )
    .bind(AUTOMATE_USERNAME_CREATION_SETTING)
    .fetch_optional(&mut *tx)
    .await?
    .and_then(|v| v.as_bool())
    .unwrap_or(true);

    // A username reserved for this address while it had no account (a workspace invite) is
    // kept, so whatever was already granted to it stays attached.
    let username = if automate_username_creation {
        let pending: Option<String> =
            sqlx::query_scalar("DELETE FROM pending_user WHERE email = $1 RETURNING username")
                .bind(&email)
                .fetch_optional(&mut *tx)
                .await?;
        match pending {
            Some(username) => Some(username),
            None => Some(generate_instance_wide_unique_username(&mut tx, &email).await?),
        }
    } else {
        None
    };

    sqlx::query(
        "INSERT INTO password (email, password_hash, login_type, super_admin, verified, name, \
         company, username) VALUES ($1, $2, $3, $4, true, $5, $6, $7)",
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(&login_type)
    .bind(nu.super_admin)
    .bind(&nu.name)
    .bind(&nu.company)
    .bind(&username)
    .execute(&mut *tx)
    .await?;

    audit_log(
        &mut *tx,
        &authed,
        "users.add_global",
        ActionKind::Create,
        "global",
        Some(&email),
        None,
    )
    .await?;

    tx.commit().await?;

    if !nu.skip_email.unwrap_or(false) {
        send_email_if_possible(
            "Your account has been created",
            &format!("An account has been created for {email} by {}.", authed.email),
            &email,
        );
    }

    Ok((StatusCode::CREATED, format!("user {email} created")))
}

pub async fn set_password(
    db: DB,
    argon2: Arc<Argon2<'_>>,
    authed: ApiAuthed,
    user_email: &str,
    ep: EditPassword,
) -> Result<String> {
    if ep.password.is_empty() {
        return Err(Error::BadRequest("password cannot be empty".to_string()));
    }

    let mut tx = db.begin().await?;

    let login_type: String =
        sqlx::query_scalar("SELECT login_type FROM password WHERE email = $1 FOR UPDATE")
            .bind(user_email)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| Error::NotFound(format!("user {user_email} not found")))?;
    if login_type == LDAP_LOGIN_TYPE {
        return Err(Error::BadRequest(format!(
            "the password of {user_email} is managed by LDAP"
        )));
    }

    let password_hash = hash_password(argon2, ep.password)?;

    // Password login only accepts `login_type = 'password'`, so a password set on a
    // `pending_oauth` or OAuth account turns it into a password account.
    sqlx::query("UPDATE password SET password_hash = $1, login_type = $2 WHERE email = $3")
        .bind(&password_hash)
        .bind(PASSWORD_LOGIN_TYPE)
        .bind(user_email)
        .execute(&mut *tx)
        .await?;

    // A password set by someone else is usually a reset: end the sessions opened with the
    // old credential. A user changing their own keeps the session they're doing it from.
    if authed.email != user_email {
        sqlx::query("DELETE FROM token WHERE email = $1 AND label = 'session'")
            .bind(user_email)
            .execute(&mut *tx)
            .await?;
    }

    audit_log(
        &mut *tx,
        &authed,
        "users.set_password",
        ActionKind::Update,
        "global",
        Some(user_email),
        None,
    )
    .await?;

    tx.commit().await?;

    Ok(format!("password of {user_email} updated"))
}

pub fn hash_password(argon2: Arc<Argon2<'_>>, password: String) -> Result<String> {
    argon2
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| Error::internal_err(format!("hashing password: {e}")))
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct OnboardingData {
    pub touch_point: String,
    pub use_case: String,
}

pub async fn submit_onboarding_data(
    _authed: ApiAuthed,
    Extension(_db): Extension<DB>,
    Json(_data): Json<OnboardingData>,
) -> Result<String> {
    Err(Error::internal_err(
        "Not implemented in Windmill's Open Source repository".to_string(),
    ))
}
