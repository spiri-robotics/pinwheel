/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2023
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */
#![allow(non_snake_case)]

use axum::{routing::post, Router};


use serde::Deserialize;



pub struct ServiceProviderExt();

pub struct SamlSsoLogin(pub Option<String>);


pub fn global_service() -> Router {
    Router::new().route("/acs", post(acs))
}

#[derive(Deserialize)]
pub struct SamlForm {
    pub SAMLResponse: Option<String>,
}


pub async fn acs() -> String {
    "SAML available only in enterprise version".to_string()
}
