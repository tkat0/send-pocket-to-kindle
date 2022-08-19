use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{
  repository::kindle::{self, KindleRepository},
  repository::pocket::{MarkAsSentInput, PocketRepository},
  repository::readability::{ConvertInput, ReadabilityRepository},
  service::send_to_kindle::{SendInput, SendOutput, SendToKindleService},
};
use anyhow::Result;
use shaku::Provider;

#[derive(Provider)]
#[shaku(interface = SendToKindleService)]
pub struct SendToKindleServiceImpl {
  #[shaku(inject)]
  kindle_repository: Arc<dyn KindleRepository>,
  #[shaku(inject)]
  pocket_repository: Arc<dyn PocketRepository>,
  #[shaku(inject)]
  readability_repository: Arc<dyn ReadabilityRepository>,
}

#[async_trait]
impl SendToKindleService for SendToKindleServiceImpl {
  async fn send(&mut self, input: SendInput) -> Result<SendOutput> {
    let ids: Vec<_> = input.articles.iter().map(|a| a.id.clone()).collect();
    self
      .pocket_repository
      .mark_as_sent(MarkAsSentInput { ids })
      .await?;

    log::info!("{:?}", &input.articles);

    let mut articles_with_content = vec![];
    for article in input.articles.into_iter() {
      let ret = self
        .readability_repository
        .convert(ConvertInput { article })
        .await?;
      articles_with_content.push(ret.article);
    }

    self
      .kindle_repository
      .send(kindle::SendInput {
        articles: articles_with_content,
      })
      .await
      .unwrap();

    Ok(SendOutput {})
  }
}
