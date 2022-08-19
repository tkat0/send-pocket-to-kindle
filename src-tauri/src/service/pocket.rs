use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{
  repository::pocket::PocketRepository,
  service::pocket::{IsLoginOutput, ListOutput, PocketService, StartLoginOutput},
};
use anyhow::Result;
use shaku::Provider;

#[derive(Provider)]
#[shaku(interface = PocketService)]
pub struct PocketServiceImpl {
  #[shaku(inject)]
  repository: Arc<dyn PocketRepository>,
}

#[async_trait]
impl PocketService for PocketServiceImpl {
  async fn is_login(&mut self) -> Result<IsLoginOutput> {
    self.repository.load_state().await?;
    let is_login = self.repository.is_login();
    Ok(IsLoginOutput { is_login })
  }

  async fn logout(&mut self) -> Result<()> {
    self.repository.logout().await?;
    Ok(())
  }

  async fn start_login(&mut self) -> Result<StartLoginOutput> {
    let ret = self.repository.start_login().await?;
    Ok(StartLoginOutput {
      auth_url: ret.auth_url,
    })
  }

  async fn list(&mut self) -> Result<ListOutput> {
    let ret = self.repository.list().await?;
    self.repository.save_state().await?;
    Ok(ListOutput {
      articles: ret.articles,
    })
  }
}
