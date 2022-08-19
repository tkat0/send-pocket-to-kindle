use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::Article;

type ItemId = String;
type ImageId = String;

#[derive(Deserialize, Debug)]
struct Item {
  item_id: ItemId,
  resolved_id: String,
  given_url: String,
  given_title: String,
  favorite: String,
  status: String,
  resolved_url: String,
  resolved_title: String,
  // The first few lines of the item (articles only)
  excerpt: String,
  is_article: String,
  has_image: String,
  has_video: String,
  word_count: String,
  tags: Option<HashMap<String, Tag>>,
  authors: Option<serde_json::Value>,
  images: Option<HashMap<ImageId, Image>>,
  videos: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct Tag {
  item_id: ItemId,
  tag: String,
}

#[derive(Deserialize, Debug)]
struct Image {
  item_id: ItemId,
  image_id: ImageId,
  src: String,
}

#[derive(Deserialize, Debug)]
pub(super) struct GetResponse {
  complete: i32,
  error: Option<String>,
  list: HashMap<ItemId, Item>,
}

#[derive(Serialize, Debug)]
pub(super) struct Action {
  #[serde(flatten)]
  pub action: ActionType,
  pub item_id: ItemId,
}

#[derive(Serialize, Debug)]
#[serde(tag = "action")]
pub(super) enum ActionType {
  add,
  archive,
  readd,
  favorite,
  unfavorite,
  delete,
  tags_add { tags: String },
  tags_remove,
  tags_replace,
  tags_clear,
  tags_rename,
  tags_delete,
}

impl Into<Vec<Article>> for GetResponse {
  fn into(self) -> Vec<Article> {
    let mut ret = vec![];
    for (_, v) in self.list.into_iter() {
      ret.push(Article {
        id: v.item_id,
        title: v.given_title,
        url: v.given_url,
        contents: v.excerpt,
        cover: v
          .images
          .as_ref()
          .and_then(|images| images.get("1"))
          .map(|image| image.src.clone()),
      })
    }
    log::info!("into: {:?}", &ret);
    ret
  }
}
