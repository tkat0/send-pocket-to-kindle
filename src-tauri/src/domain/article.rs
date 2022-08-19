use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub type ArticleId = String;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Article {
  pub id: ArticleId,
  pub title: String,
  pub url: String,
  pub cover: Option<String>,
  pub contents: String,
}
