use async_trait::async_trait;

use crate::domain::Article;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[async_trait]
pub trait SendToKindleService: Send {
  async fn send(&mut self, input: SendInput) -> Result<SendOutput>;
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SendInput {
  pub articles: Vec<Article>,
}

// #[derive(Serialize, TS)]
#[derive(Serialize)]
// #[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SendOutput {}
