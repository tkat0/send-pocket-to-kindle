use std::fs::create_dir_all;

use anyhow::{Context, Result};
use shaku::module;
use tauri::{App, Manager};

use crate::{
  domain::repository::{pocket::PocketRepository, readability::ReadabilityRepository},
  env::Env,
  repository::{
    kindle::{KindleRepositoryConfig, KindleRepositoryImpl, KindleRepositoryImplParameters},
    pocket::{PocketRepositoryConfig, PocketRepositoryImpl},
    readability::ReadabilityRepositoryImpl,
  },
  service::{pocket::PocketServiceImpl, send_to_kindle::SendToKindleServiceImpl},
};

module! {
    pub MyModule {
        components = [PocketRepositoryImpl, ReadabilityRepositoryImpl, KindleRepositoryImpl],
        providers = [PocketServiceImpl, SendToKindleServiceImpl]
    }
}

pub struct AppState {
  pub module: MyModule,
}

impl AppState {
  pub fn new(app: &App) -> Result<Self> {
    let env = envy::from_env::<Env>()?;

    let app_dir = app.path_resolver().app_dir().context("app_dir not found")?;
    log::debug!("app_dir: {:?}", &app_dir);

    create_dir_all(&app_dir)?;

    let state_path = app_dir.join(".pocket-repository-state");

    let module = MyModule::builder()
      .with_component_override::<dyn PocketRepository>(Box::new(PocketRepositoryImpl::new(
        app.app_handle(),
        PocketRepositoryConfig {
          consumer_key: env.pocket_platform_consumer_key,
          state_file_path: state_path.to_str().context("parse path fails")?.into(),
        },
      )))
      .with_component_override::<dyn ReadabilityRepository>(Box::new(
        ReadabilityRepositoryImpl::new(app.app_handle())?,
      ))
      .with_component_parameters::<KindleRepositoryImpl>(KindleRepositoryImplParameters {
        config: KindleRepositoryConfig {
          send_to: env.send_to_kindle_email,
          send_from: env.email_user,
          password: env.email_password,
        },
      })
      .build();

    Ok(Self { module })
  }
}
