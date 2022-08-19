use anyhow::Result;
use async_trait::async_trait;
use shaku::Interface;

use crate::domain::Article;

pub struct ConvertInput {
  pub article: Article,
}

pub struct ConvertOutput {
  pub article: Article,
}

#[async_trait]
pub trait ReadabilityRepository: Interface {
  async fn convert(&self, input: ConvertInput) -> Result<ConvertOutput>;
}
