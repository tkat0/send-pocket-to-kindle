use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Env {
  pub pocket_platform_consumer_key: String,
  pub send_to_kindle_email: String,
  pub email_user: String,
  pub email_password: String,
}
