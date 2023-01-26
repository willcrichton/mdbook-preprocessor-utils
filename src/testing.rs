use anyhow::Result;
use mdbook::{
  book::{load_book, Book},
  config::BuildConfig,
  preprocess::{CmdPreprocessor, Preprocessor},
  MDBook,
};
use std::{env, path::Path};
use tempfile::{tempdir, TempDir};

use crate::{processor::SimplePreprocessorDriver, SimplePreprocessor};

pub struct MdbookTestHarness {
  dir: TempDir,
}

impl MdbookTestHarness {
  pub fn new() -> Result<Self> {
    let dir = tempdir()?;
    let builder = MDBook::init(dir.path());
    builder.build()?;
    Ok(MdbookTestHarness { dir })
  }

  pub fn root(&self) -> &Path {
    self.dir.path()
  }

  pub fn compile<P: SimplePreprocessor>(&self, config: serde_json::Value) -> Result<Book> {
    let book = load_book(self.root().join("src"), &BuildConfig::default())?;
    let json = serde_json::json!(
      [
        {
          "root": self.root().display().to_string(),
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

    env::set_current_dir(self.root())?;

    let preprocessor = SimplePreprocessorDriver::<P>::new();
    let (ctx, book) = CmdPreprocessor::parse_input(json_str.as_bytes())?;
    let book = preprocessor.run(&ctx, book)?;

    Ok(book)
  }
}
