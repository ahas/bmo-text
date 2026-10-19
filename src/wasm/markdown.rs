use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use super::{answer, argument, count, list, string, units_at};
use crate::js::chars;
use crate::markdown::{
  Block, Converted, Document, Fold, Inline, Pair, Span, code_markup, convert_markdown, fold_blocks,
  folded_markup, markdown_parts, parse_inline, parse_markdown, plain_inline, render_inline,
};
use crate::telegram::TEXT_LIMIT;

fn pair_kind(pair: Pair) -> &'static str {
  match pair {
    Pair::Bold => "b",
    Pair::Italic => "i",
    Pair::Strike => "s",
    Pair::Spoiler => "spoiler",
  }
}

fn inline_json(inline: &Inline) -> Value {
  match inline {
    Inline::Text(text) => json!({ "kind": "text", "text": text }),
    Inline::Code(text) => json!({ "kind": "code", "text": text }),
    Inline::Pair(pair, children) => json!({ "kind": pair_kind(*pair), "children": line_json(children) }),
    Inline::Link(url, children) => json!({ "kind": "link", "url": url, "children": line_json(children) }),
  }
}

fn line_json(line: &[Inline]) -> Value {
  line.iter().map(inline_json).collect()
}

fn inline_of(value: &Value) -> Result<Inline, JsError> {
  let children = || line_of(&value["children"]);

  Ok(match string(value, "kind")?.as_str() {
    "text" => Inline::Text(string(value, "text")?),
    "code" => Inline::Code(string(value, "text")?),
    "b" => Inline::Pair(Pair::Bold, children()?),
    "i" => Inline::Pair(Pair::Italic, children()?),
    "s" => Inline::Pair(Pair::Strike, children()?),
    "spoiler" => Inline::Pair(Pair::Spoiler, children()?),
    "link" => Inline::Link(string(value, "url")?, children()?),
    other => return Err(JsError::new(&format!("No piece of a line is of the kind {other}."))),
  })
}

fn line_of(value: &Value) -> Result<Vec<Inline>, JsError> {
  list(value, inline_of)
}

fn block_json(block: &Block) -> Value {
  match block {
    Block::Lines(lines) => {
      json!({ "kind": "lines", "lines": lines.iter().map(|line| line_json(line)).collect::<Vec<_>>() })
    }
    Block::Pre { language, text } => json!({ "kind": "pre", "language": language, "text": text }),
    Block::Quote(document) => {
      json!({ "kind": "quote", "blocks": blocks_json(&document.blocks), "gaps": document.gaps })
    }
  }
}

fn blocks_json(blocks: &[Block]) -> Vec<Value> {
  blocks.iter().map(block_json).collect()
}

fn block_of(value: &Value) -> Result<Block, JsError> {
  Ok(match string(value, "kind")?.as_str() {
    "lines" => Block::Lines(list(&value["lines"], line_of)?),
    "pre" => Block::Pre {
      language: value["language"].as_str().map(str::to_owned),
      text: string(value, "text")?,
    },
    "quote" => Block::Quote(document_of(value)?),
    other => return Err(JsError::new(&format!("No block is of the kind {other}."))),
  })
}

fn document_json(document: &Document) -> Value {
  json!({ "blocks": blocks_json(&document.blocks), "gaps": document.gaps })
}

fn gaps_of(value: &Value) -> Result<Vec<String>, JsError> {
  list(value, |gap| {
    gap
      .as_str()
      .map(str::to_owned)
      .ok_or_else(|| JsError::new("A gap between blocks is not a string."))
  })
}

fn document_of(value: &Value) -> Result<Document, JsError> {
  Ok(Document {
    blocks: list(&value["blocks"], block_of)?,
    gaps: gaps_of(&value["gaps"])?,
  })
}

fn converted_of(value: &Value) -> Result<Converted, JsError> {
  Ok(Converted {
    source: string(value, "source")?,
    document: document_of(&value["document"])?,
    markup: string(value, "markup")?,
    plain: string(value, "plain")?,
    formatted: value["formatted"].as_bool().unwrap_or_default(),
  })
}

fn fold_json(fold: &Fold) -> Value {
  json!({
    "opening": fold.opening.as_ref().map(block_json),
    "openingGap": fold.opening_gap,
    "hidden": blocks_json(&fold.hidden),
    "hiddenGaps": fold.hidden_gaps,
    "closing": fold.closing.as_ref().map(block_json),
    "closingGap": fold.closing_gap,
  })
}

/// A line's pieces, and where each construct stood in UTF-16 units where `spans` asks for it.
#[wasm_bindgen(js_name = parseInline)]
pub fn parse_inline_of(text: &str, spans: bool) -> JsValue {
  let characters = chars(text);
  let mut found: Vec<Span> = vec![];
  let line = parse_inline(&characters, spans.then_some(&mut found));
  let found: Vec<Value> = found
    .iter()
    .map(|span| {
      json!({ "start": units_at(&characters, span.start), "end": units_at(&characters, span.end), "literal": span.literal })
    })
    .collect();

  answer(&json!({ "line": line_json(&line), "spans": found }))
}

#[wasm_bindgen(js_name = parseMarkdown)]
pub fn parse_markdown_of(source: &str) -> JsValue {
  answer(&document_json(&parse_markdown(source)))
}

#[wasm_bindgen(js_name = renderInline)]
pub fn render_inline_of(nodes: &str) -> Result<String, JsError> {
  Ok(render_inline(&line_of(&argument(nodes)?)?))
}

#[wasm_bindgen(js_name = plainInline)]
pub fn plain_inline_of(nodes: &str) -> Result<String, JsError> {
  Ok(plain_inline(&line_of(&argument(nodes)?)?))
}

#[wasm_bindgen(js_name = convertMarkdown)]
pub fn convert_markdown_of(source: &str) -> JsValue {
  let converted = convert_markdown(source);

  answer(&json!({
    "source": converted.source,
    "document": document_json(&converted.document),
    "markup": converted.markup,
    "plain": converted.plain,
    "formatted": converted.formatted,
  }))
}

#[wasm_bindgen(js_name = foldBlocks)]
pub fn fold_blocks_of(document: &str) -> Result<JsValue, JsError> {
  let fold = fold_blocks(&document_of(&argument(document)?)?);

  Ok(fold.as_ref().map_or(JsValue::NULL, |fold| answer(&fold_json(fold))))
}

#[wasm_bindgen(js_name = foldedMarkup)]
pub fn folded_markup_of(converted: &str) -> Result<Option<String>, JsError> {
  Ok(folded_markup(&converted_of(&argument(converted)?)?))
}

#[wasm_bindgen(js_name = codeMarkup)]
pub fn code_markup_of(converted: &str) -> Result<Option<String>, JsError> {
  Ok(code_markup(&converted_of(&argument(converted)?)?))
}

#[wasm_bindgen(js_name = markdownParts)]
pub fn markdown_parts_of(text: &str, limit: Option<f64>) -> Vec<String> {
  markdown_parts(text, limit.map_or(TEXT_LIMIT, count))
}
