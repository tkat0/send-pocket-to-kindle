use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;

use crate::domain::Article;

pub struct SendInput {
  pub articles: Vec<Article>,
}

#[async_trait]
pub trait KindleRepository: Interface {
  async fn send(&self, input: SendInput) -> Result<()>;
}
