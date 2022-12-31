use anyhow::Result;
use mdbook::{
  book::{load_book, Book},
  config::BuildConfig,
  preprocess::{CmdPreprocessor, Preprocessor},
  MDBook,
};
use std::path::PathBuf;
use tempfile::tempdir;

use crate::{processor::SimplePreprocessorDriver, SimplePreprocessor};

pub struct MdbookTestHarness {
  pub dir: PathBuf,
}

impl MdbookTestHarness {
  pub fn new() -> Result<Self> {
    let dir = tempdir()?.into_path();
    let builder = MDBook::init(&dir);
    builder.build()?;
    Ok(MdbookTestHarness { dir })
  }

  pub fn compile<P: SimplePreprocessor>(&self, config: serde_json::Value) -> Result<Book> {
    let book = load_book(self.dir.join("src"), &BuildConfig::default())?;
    let json = serde_json::json!(
      [
        {
          "root": self.dir.display().to_string(),
          "config": {
            "preprocessor": {
              P::name(): config,
            },
          },
          "renderer": "html",
          "mdbook_version": "0.1.0"
        },
        serde_json::to_value(&book)?
      ]
    );
    let json_str = serde_json::to_string(&json)?;

    let preprocessor = SimplePreprocessorDriver::<P>::new();
    let (ctx, book) = CmdPreprocessor::parse_input(json_str.as_bytes())?;
    let book = preprocessor.run(&ctx, book)?;

    Ok(book)
  }
}
