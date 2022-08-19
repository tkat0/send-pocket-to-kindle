use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use shaku::Component;
use std::{
  collections::HashMap,
  fs::{remove_file, File},
  io::{Read, Write},
  net::{TcpListener, TcpStream},
  sync::{Arc, Mutex},
  thread::{self, JoinHandle},
};
use tauri::{AppHandle, Manager};
use tokio::fs::read_to_string;

use crate::domain::repository::pocket::{
  ListOutput, MarkAsSentInput, PocketRepository, StartLoginOutput, WaitLoginOutput,
};
use data::*;

mod data;

const POCKET_API_AUTHORIZE: &str = "https://getpocket.com/auth/authorize";
const POCKET_API_OAUTH_REQUEST: &str = "https://getpocket.com/v3/oauth/request";
const POCKET_API_OAUTH_AUTHORIZE: &str = "https://getpocket.com/v3/oauth/authorize";
const POCKET_API_GET: &str = "https://getpocket.com/v3/get";
const POCKET_API_MODIFY: &str = "https://getpocket.com/v3/send";

#[derive(Serialize, Deserialize, Debug)]
pub struct PocketRepositoryState {
  pub access_token: String,
}

#[derive(Default)]
pub struct PocketRepositoryConfig {
  pub consumer_key: String,
  pub state_file_path: String,
}

#[derive(Component)]
#[shaku(interface = PocketRepository)]
pub struct PocketRepositoryImpl {
  config: PocketRepositoryConfig,
  handle: Mutex<Option<JoinHandle<Result<String>>>>,
  access_token: Arc<Mutex<Option<String>>>,
  app: AppHandle,
}

#[async_trait]
impl PocketRepository for PocketRepositoryImpl {
  fn is_login(&self) -> bool {
    self.access_token.lock().unwrap().is_some()
  }

  async fn load_state(&self) -> Result<()> {
    if let Ok(state) = read_to_string(&self.config.state_file_path).await {
      let state: PocketRepositoryState = serde_json::from_str(&state).unwrap();

      log::info!("load_state: {:?}", &state);

      {
        let mut token = self.access_token.lock().unwrap();
        *token = Some(state.access_token.clone());
      }
    }
    Ok(())
  }

  async fn save_state(&self) -> Result<()> {
    let token = self.access_token.lock().unwrap();
    let access_token = token.as_ref().unwrap();

    // ~/Library/Application Support/com.example.tauri-template/.pocket-repository-state
    log::info!("save_state: {}", &self.config.state_file_path);

    serde_json::to_writer(
      File::create(&self.config.state_file_path)?,
      &PocketRepositoryState {
        access_token: access_token.clone(),
      },
    )?;

    Ok(())
  }

  async fn start_login(&self) -> Result<StartLoginOutput> {
    log::info!("login");
    let client = reqwest::Client::new();

    let redirect_uri = "http://127.0.0.1:8080";

    let mut map = HashMap::new();
    map.insert("consumer_key", self.config.consumer_key.as_str());
    map.insert("redirect_uri", redirect_uri);

    let res: serde_json::Value = client
      .post(POCKET_API_OAUTH_REQUEST)
      .header("X-ACCEPT", HeaderValue::from_static("application/json"))
      .json(&map)
      .send()
      .await?
      .json()
      .await?;

    log::debug!("{:?}", &res);

    let code = res
      .get("code")
      .and_then(serde_json::Value::as_str)
      .context("failed to get token")?;

    self.start_server_for_callback(code)?;

    Ok(StartLoginOutput {
      auth_url: format!(
        "{}?request_token={}&redirect_uri={}",
        POCKET_API_AUTHORIZE, code, redirect_uri
      ),
    })
  }

  async fn wait_login(&self) -> Result<WaitLoginOutput> {
    log::info!("wait_login");
    {
      let token = self.access_token.lock().unwrap();
      if let Some(token) = token.as_ref() {
        return Ok(WaitLoginOutput {
          access_token: token.into(),
        });
      }
    }

    let mut handle = self.handle.lock().unwrap();

    let access_token = handle
      .take()
      .context("call start_login first")?
      .join()
      .unwrap()?;

    log::info!("wait_login ok");

    {
      let mut token = self.access_token.lock().unwrap();
      *token = Some(access_token.clone());
    }

    Ok(WaitLoginOutput { access_token })
  }

  async fn logout(&self) -> Result<()> {
    remove_file(&self.config.state_file_path)?;
    Ok(())
  }

  async fn list(&self) -> Result<ListOutput> {
    self.wait_login().await?; // TODO: move
    let access_token = self.get_access_token()?;

    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("consumer_key", self.config.consumer_key.as_str());
    map.insert("access_token", &access_token);
    map.insert("detailType", "complete");
    map.insert("sort", "newest");

    let res: GetResponse = client
      .post(POCKET_API_GET)
      .header("X-ACCEPT", HeaderValue::from_static("application/json"))
      .json(&map)
      .send()
      .await?
      .json()
      .await?;

    log::debug!("res: {:?}", &res);

    Ok(ListOutput {
      articles: res.into(),
    })
  }

  async fn mark_as_sent(&self, input: MarkAsSentInput) -> Result<()> {
    let access_token = self.get_access_token()?;

    let actions: Vec<Action> = input
      .ids
      .into_iter()
      .map(|id| Action {
        action: ActionType::tags_add {
          tags: "sent-to-kindle".into(),
        },
        item_id: id,
      })
      .collect();

    let actions = serde_json::to_string(&actions)?;

    let mut map = HashMap::new();
    map.insert("consumer_key", self.config.consumer_key.as_str());
    map.insert("access_token", &access_token);
    map.insert("actions", &actions);

    let client = reqwest::Client::new();

    let res: serde_json::Value = client
      .post(POCKET_API_MODIFY)
      .header("X-ACCEPT", HeaderValue::from_static("application/json"))
      .json(&map)
      .send()
      .await?
      .json()
      .await?;

    log::debug!("{:?}", &res);

    Ok(())
  }
}

impl PocketRepositoryImpl {
  pub fn new(app: AppHandle, config: PocketRepositoryConfig) -> Self {
    Self {
      access_token: Arc::new(Mutex::new(None)),
      config,
      handle: Mutex::new(None),
      app,
    }
  }

  fn start_server_for_callback(&self, code: &str) -> Result<()> {
    let code = code.to_string();
    let consumer_key = self.config.consumer_key.clone();

    let app = self.app.clone();

    let handle = thread::spawn(move || -> Result<String> {
      let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

      for stream in listener.incoming() {
        match stream {
          Ok(stream) => {
            handler(stream);
            break;
          }
          Err(e) => {
            println!("Unable to connect: {}", e);
          }
        }
      }

      // get access token
      log::info!("get access token");
      let client = reqwest::blocking::Client::new();

      let mut map = HashMap::new();
      map.insert("consumer_key", &consumer_key);
      map.insert("code", &code);

      let res: serde_json::Value = client
        .post(POCKET_API_OAUTH_AUTHORIZE)
        .header("X-ACCEPT", HeaderValue::from_static("application/json"))
        .json(&map)
        .send()?
        .json()?;

      let access_token = res
        .get("access_token")
        .and_then(serde_json::Value::as_str)
        .context("failed to get access_token")?;

      app.emit_all("login", ()).expect("emit login event failed");

      Ok(access_token.into())
    });

    {
      let mut h = self.handle.lock().unwrap();
      *h = Some(handle);
    }

    Ok(())
  }

  fn get_access_token(&self) -> Result<String> {
    let access_token = self.access_token.lock().unwrap();
    access_token
      .as_ref()
      .context("access_token not found")
      .map(|s| s.clone())
  }
}

fn handler(mut stream: TcpStream) {
  let mut buf = [0u8; 4096];
  match stream.read(&mut buf) {
    Ok(_) => {
      let req_str = String::from_utf8_lossy(&buf);
      println!("{}", req_str);
    }
    Err(e) => println!("Unable to read stream: {}", e),
  }

  let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body>Close this window</body></html>\r\n";
  match stream.write(response) {
    Ok(_) => println!("Response sent"),
    Err(e) => println!("Failed sending response: {}", e),
  }
}
