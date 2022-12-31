use std::{
  fs,
  marker::PhantomData,
  ops::Range,
  path::{Path, PathBuf},
};

use anyhow::Result;
use mdbook::{
  book::Book,
  preprocess::{Preprocessor, PreprocessorContext},
  BookItem,
};

#[derive(Copy, Clone)]
pub struct Asset {
  pub name: &'static str,
  pub contents: &'static [u8],
}

#[macro_export]
macro_rules! asset_generator {
  ($base:expr) => {
    macro_rules! make_asset {
      ($name:expr) => {
        $crate::Asset {
          name: $name,
          contents: include_bytes!(concat!($base, $name)),
        }
      };
    }
  };
}

pub trait SimplePreprocessor: Sized + Send + Sync {
  fn name() -> &'static str;
  fn build(ctx: &PreprocessorContext) -> Result<Self>;
  fn replacements(&self, chapter_dir: &Path, content: &str) -> Result<Vec<(Range<usize>, String)>>;
  fn linked_assets(&self) -> Vec<Asset>;
  fn all_assets(&self) -> Vec<Asset>;
}

struct SimplePreprocessorDriverCtxt<P: SimplePreprocessor> {
  sp: P,
  src_dir: PathBuf,
}

impl<P: SimplePreprocessor> SimplePreprocessorDriverCtxt<P> {
  fn copy_assets(&self) -> Result<()> {
    // Rather than copying directly to the build directory, we instead copy to the book source
    // since mdBook will clean the build-dir after preprocessing. See mdBook#1087 for more.
    let dst_dir = self.src_dir.join(P::name());
    fs::create_dir_all(&dst_dir)?;

    for asset in self.sp.all_assets() {
      fs::write(dst_dir.join(asset.name), asset.contents)?;
    }

    Ok(())
  }

  fn process_chapter(&self, chapter_dir: &Path, content: &mut String) -> Result<()> {
    let replacements = self.sp.replacements(chapter_dir, content)?;
    if !replacements.is_empty() {
      for (range, html) in replacements.into_iter().rev() {
        content.replace_range(range, &html);
      }

      // If a chapter is located at foo/bar/the_chapter.md, then the generated source files
      // will be at foo/bar/the_chapter.html. So they need to reference preprocessor files
      // at ../../<preprocessor>/embed.js, i.e. we generate the right number of "..".
      let chapter_rel_path = chapter_dir.strip_prefix(&self.src_dir).unwrap();
      let depth = chapter_rel_path.components().count();
      let prefix = vec![".."; depth].into_iter().collect::<PathBuf>();

      // Ensure there's space between existing markdown and inserted HTML
      content.push_str("\n\n");

      for asset in self.sp.linked_assets() {
        let asset_rel = prefix.join(P::name()).join(asset.name);
        let asset_str = asset_rel.display().to_string();
        let link = match &*asset_rel.extension().unwrap().to_string_lossy() {
          "js" => format!(r#"<script type="text/javascript" src="{asset_str}"></script>"#),
          "css" => format!(r#"<link rel="stylesheet" type="text/css" href="{asset_str}">"#),
          _ => continue,
        };
        content.push_str(&link);
      }
    }
    Ok(())
  }
}

pub(crate) struct SimplePreprocessorDriver<P: SimplePreprocessor>(PhantomData<P>);

impl<P: SimplePreprocessor> SimplePreprocessorDriver<P> {
  pub fn new() -> Self {
    SimplePreprocessorDriver(PhantomData)
  }
}

impl<P: SimplePreprocessor> Preprocessor for SimplePreprocessorDriver<P> {
  fn name(&self) -> &str {
    P::name()
  }

  fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
    let src_dir = ctx.root.join(&ctx.config.book.src);
    let sp = P::build(ctx)?;
    let ctxt = SimplePreprocessorDriverCtxt { sp, src_dir };
    ctxt.copy_assets()?;

    // Limit size of thread pool to avoid OS resource exhaustion
    let nproc = std::thread::available_parallelism().map_or(1, |n| n.get());
    rayon::ThreadPoolBuilder::new()
      .num_threads(nproc)
      .build_global()
      .unwrap();

    rayon::scope(|s| {
      fn for_each_mut<'scope, 'proc: 'scope, 'item: 'scope, P: SimplePreprocessor>(
        s: &rayon::Scope<'scope>,
        ctxt: &'proc SimplePreprocessorDriverCtxt<P>,
        items: impl IntoIterator<Item = &'item mut BookItem>,
      ) {
        for item in items {
          if let BookItem::Chapter(chapter) = item {
            if chapter.path.is_some() {
              s.spawn(|_| {
                let chapter_path_abs = ctxt.src_dir.join(chapter.path.as_ref().unwrap());
                let chapter_dir = chapter_path_abs.parent().unwrap();
                ctxt
                  .process_chapter(chapter_dir, &mut chapter.content)
                  .unwrap();
              });
              for_each_mut(s, ctxt, &mut chapter.sub_items);
            }
          }
        }
      }

      for_each_mut(s, &ctxt, &mut book.sections);
    });

    Ok(book)
  }
}
