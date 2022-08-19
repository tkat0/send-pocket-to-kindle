use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use crate::domain::repository::kindle::{KindleRepository, SendInput};
use crate::domain::Article;
use anyhow::Result;
use async_trait::async_trait;
use epub_builder::EpubBuilder;
use epub_builder::EpubContent;
use epub_builder::ReferenceType;
use epub_builder::TocElement;
use epub_builder::ZipLibrary;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use shaku::Component;

#[derive(Default)]
pub struct KindleRepositoryConfig {
  // pub save_dir: PathBuf,
  pub send_to: String,
  pub send_from: String,
  pub password: String,
}

#[derive(Component)]
#[shaku(interface = KindleRepository)]
pub struct KindleRepositoryImpl {
  #[shaku(default = KindleRepositoryConfig::default())]
  config: KindleRepositoryConfig,
}

#[async_trait]
impl KindleRepository for KindleRepositoryImpl {
  async fn send(&self, input: SendInput) -> Result<()> {
    log::info!("send {} articles", input.articles.len());

    // let path = self.config.save_dir.join("test.txt");
    /*
    let mut attachiments: Vec<SinglePart> = input
      .articles
      .into_iter()
      .map(|article| {
        let content = article.contents;

        let content = self.create_epub(&article).unwrap();

        let content_type = ContentType::parse("text/plain").unwrap();
        Attachment::new(article.title).body(content, content_type)
      })
      .collect();
     */

    let content = self.create_epub(&input.articles)?;

    // let mut f = File::create("test.epub")?;
    // f.write(&content)?;

    let content_type = ContentType::parse("application/epub+zip").unwrap();
    let mut attachiments =
      vec![Attachment::new("pocket.epub".to_string()).body(content, content_type)];

    let mut part = MultiPart::alternative().singlepart(attachiments.pop().unwrap());

    for attachment in attachiments {
      part = part.singlepart(attachment);
    }

    // maximum: 25 items, 50 MB
    // supported file types: .doc, .html, .txt, .pdf, .epub

    let email = Message::builder()
      .from(self.config.send_from.parse().unwrap())
      .to(self.config.send_to.parse().unwrap())
      .subject("Send Pocket articles to Kindle")
      .multipart(part)?;

    let creds = Credentials::new(self.config.send_from.clone(), self.config.password.clone());

    let mailer = SmtpTransport::relay("smtp.gmail.com")
      .unwrap()
      .credentials(creds)
      .build();

    match mailer.send(&email) {
      Ok(_) => log::info!("Email sent successfully!"),
      Err(e) => panic!("Could not send email: {:?}", e),
    }

    Ok(())
  }
}

impl KindleRepositoryImpl {
  fn create_epub(&self, articles: &[Article]) -> Result<Vec<u8>> {
    let mut epub: Vec<u8> = vec![];

    let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();

    let builder = builder
      .metadata("title", "Pocket")
      .unwrap()
      .metadata("author", "tkat0")
      .unwrap()
      .inline_toc();

    for article in articles {
      builder
        .add_content(
          EpubContent::new(
            &format!("{}.xhtml", &article.id),
            format!(
              r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<body>
{}
</body>
</html>"#,
              article.contents
            )
            .as_bytes(),
          )
          .title(&article.title)
          .reftype(ReferenceType::Text),
        )
        .unwrap();
    }

    builder.generate(&mut epub).unwrap();

    Ok(epub)
  }
}
