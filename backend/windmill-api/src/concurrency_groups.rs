use crate::db::{ApiAuthed, DB};
use std::collections::HashMap;

use axum::extract::Path;
use axum::routing::{delete, get};
use axum::{Extension, Json, Router};
use polars::prelude::IntoVec;
use serde::Serialize;
use windmill_common::error::Error::{InternalErr, PermissionDenied};
use windmill_common::error::JsonResult;


pub fn global_service() -> Router {
    Router::new()
}



