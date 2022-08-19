use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;

use crate::domain::{Article, ArticleId};

pub struct MarkAsSentInput {
  pub ids: Vec<ArticleId>,
}

pub struct StartLoginOutput {
  pub auth_url: String,
}

pub struct WaitLoginOutput {
  pub access_token: String,
}

pub struct ListOutput {
  pub articles: Vec<Article>,
}

#[async_trait]
pub trait PocketRepository: Interface {
  fn is_login(&self) -> bool;
  async fn load_state(&self) -> Result<()>;
  async fn save_state(&self) -> Result<()>;
  async fn start_login(&self) -> Result<StartLoginOutput>;
  async fn wait_login(&self) -> Result<WaitLoginOutput>;
  async fn logout(&self) -> Result<()>;
  async fn list(&self) -> Result<ListOutput>;
  async fn mark_as_sent(&self, input: MarkAsSentInput) -> Result<()>;
}
