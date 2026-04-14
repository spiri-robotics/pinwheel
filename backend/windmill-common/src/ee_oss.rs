
use crate::ee_oss::LicensePlan::Community;
use serde::Deserialize;
use std::sync::atomic::AtomicBool;

lazy_static::lazy_static! {
  pub static ref LICENSE_KEY_VALID: AtomicBool = AtomicBool::new(true);
  pub static ref LICENSE_KEY_ID: arc_swap::ArcSwap<String> = arc_swap::ArcSwap::from_pointee("".to_string());
  pub static ref LICENSE_KEY: arc_swap::ArcSwap<String> = arc_swap::ArcSwap::from_pointee("".to_string());
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








