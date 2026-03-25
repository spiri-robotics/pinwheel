
use crate::ee_oss::LicensePlan::Community;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
  pub static ref LICENSE_KEY_VALID: Arc<RwLock<bool>> = Arc::new(RwLock::new(true));
  pub static ref LICENSE_KEY_ID: Arc<RwLock<String>> = Arc::new(RwLock::new("".to_string()));
  pub static ref LICENSE_KEY: Arc<RwLock<String>> = Arc::new(RwLock::new("".to_string()));
}

#[derive(PartialEq, Eq)]
pub enum LicensePlan {
    Community,
    Pro,
    Enterprise,
}

pub async fn get_license_plan() -> LicensePlan {
    // Implementation is not open source
    return Community;
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum CriticalErrorChannel {
    Email { email: String },
    Slack { slack_channel: String },
    Teams { teams_channel: TeamsChannel },
}

#[derive(Deserialize)]
pub struct TeamsChannel {
    pub team_id: String,
    pub team_name: String,
    pub channel_id: String,
    pub channel_name: String,
}

pub enum CriticalAlertKind {
}








