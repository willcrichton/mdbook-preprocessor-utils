use anyhow::Result;
use serde::Serialize;
use std::fmt::Write;

pub struct HtmlElementBuilder {
  html: String,
}

impl HtmlElementBuilder {
  pub fn new() -> Self {
    let html = String::from("<div");
    HtmlElementBuilder { html }
  }

  pub fn attr(&mut self, key: &str, value: &str) -> Result<&mut Self> {
    let value_escaped = html_escape::encode_double_quoted_attribute(&value);
    write!(&mut self.html, " {key}=\"{value_escaped}\"")?;
    Ok(self)
  }

  pub fn data(&mut self, key: &str, value: impl Serialize) -> Result<&mut Self> {
    let value_json = serde_json::to_string(&value)?;
    let value_escaped = html_escape::encode_double_quoted_attribute(&value_json);
    write!(&mut self.html, " data-{key}=\"{value_escaped}\"",)?;
    Ok(self)
  }

  pub fn finish(mut self) -> Result<String> {
    write!(&mut self.html, "></div>")?;
    Ok(self.html)
  }
}
