use anyhow::Result;
use async_trait::async_trait;
use crossbeam_channel::{bounded, Receiver};
use serde::{Deserialize, Serialize};
use shaku::Component;
use tauri::{AppHandle, Manager};

use crate::domain::{
  repository::readability::{ConvertInput, ConvertOutput, ReadabilityRepository},
  Article,
};

const EVENT_REQUEST: &str = "readability-request";
const EVENT_RESPONSE: &str = "readability-response";

#[derive(Serialize, Clone, Debug)]
struct Request {
  pub content: String,
}

#[derive(Deserialize, Debug)]
struct ReadabilityOutput {
  pub title: String,
  pub content: String,
  pub textContent: String,
  pub length: usize,
  pub excerpt: String,
  pub byline: Option<String>,
  pub dir: Option<String>,
  pub siteName: Option<String>,
  pub lang: String,
}

#[derive(Deserialize, Debug)]
struct Response {
  pub article: ReadabilityOutput,
}

#[derive(Default)]
pub struct ReadabilityRepositoryConfig {
  pub consumer_key: String,
}

#[derive(Component)]
#[shaku(interface = ReadabilityRepository)]
pub struct ReadabilityRepositoryImpl {
  config: ReadabilityRepositoryConfig,
  app: AppHandle,
  rx: Receiver<String>,
}

#[async_trait]
impl ReadabilityRepository for ReadabilityRepositoryImpl {
  async fn convert(&self, input: ConvertInput) -> Result<ConvertOutput> {
    let client = reqwest::Client::new();
    let content = client.get(&input.article.url).send().await?.text().await?;

    // call `Readability.js` from Rust

    // -> send event
    let payload = Request { content };
    self.app.emit_all(EVENT_REQUEST, payload)?;

    log::info!("waiting for response...");

    // <- reveive event
    // NOTE: use async command of tauri as blocking operation blocks webview...
    let ret = self.rx.recv()?;
    let ret: Response = serde_json::from_str(&ret)?;

    // log::info!("-> {:?}", &ret);

    Ok(ConvertOutput {
      article: Article {
        id: input.article.id,
        title: input.article.title,
        url: input.article.url,
        cover: input.article.cover,
        // contents: ret.article.textContent,
        contents: ret.article.content,
      },
    })
  }
}

impl ReadabilityRepositoryImpl {
  pub fn new(app: AppHandle) -> Result<Self> {
    let (tx, rx) = bounded::<String>(1);

    // set callback
    app.listen_global(EVENT_RESPONSE, move |event| {
      // TODO: how to handle the error inside listener?
      let p = event.payload().unwrap();
      // log::info!("response: {:?}", &p);
      tx.send(p.into()).unwrap();
    });

    Ok(Self {
      config: ReadabilityRepositoryConfig::default(),
      app,
      rx,
    })
  }
}
