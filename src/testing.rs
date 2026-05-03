use anyhow::Result;
use mdbook_driver::MDBook;
use mdbook_preprocessor::{Preprocessor, book::Book};
use std::{env, path::Path};
use tempfile::{TempDir, tempdir};

use crate::{SimplePreprocessor, processor::SimplePreprocessorDriver};

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
    let book = MDBook::load(self.root())?;
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
        serde_json::to_value(&book.book)?
      ]
    );
    let json_str = serde_json::to_string(&json)?;

    env::set_current_dir(self.root())?;

    let preprocessor = SimplePreprocessorDriver::<P>::new();
    let (ctx, book) = mdbook_preprocessor::parse_input(json_str.as_bytes())?;
    let book = preprocessor.run(&ctx, book)?;

    Ok(book)
  }
}
