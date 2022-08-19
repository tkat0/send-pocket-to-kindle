use async_trait::async_trait;

use crate::domain::Article;
use anyhow::Result;
use serde::Serialize;
use ts_rs::TS;

#[async_trait]
pub trait PocketService: Send {
  async fn is_login(&mut self) -> Result<IsLoginOutput>;
  async fn logout(&mut self) -> Result<()>;
  async fn start_login(&mut self) -> Result<StartLoginOutput>;
  async fn list(&mut self) -> Result<ListOutput>;
}

#[derive(Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct IsLoginOutput {
  pub is_login: bool,
}

#[derive(Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct StartLoginOutput {
  pub auth_url: String,
}

#[derive(Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ListOutput {
  pub articles: Vec<Article>,
}
