//! The markdown reading held to the cases the package's is held to, its reading written in the package's
//! shape: a piece of a line by its kind, a block by its kind, a fold by its parts.

mod shared;

use bmo_markup::js::chars;
use bmo_markup::markdown::*;
use bmo_markup::telegram::TEXT_LIMIT;
use serde_json::{Value, json};

use shared::text;

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

fn inline_of(value: &Value) -> Inline {
  let children = || line_of(&value["children"]);

  match text(&value["kind"]) {
    "text" => Inline::Text(text(&value["text"]).to_owned()),
    "code" => Inline::Code(text(&value["text"]).to_owned()),
    "b" => Inline::Pair(Pair::Bold, children()),
    "i" => Inline::Pair(Pair::Italic, children()),
    "s" => Inline::Pair(Pair::Strike, children()),
    "spoiler" => Inline::Pair(Pair::Spoiler, children()),
    "link" => Inline::Link(text(&value["url"]).to_owned(), children()),
    other => panic!("no piece of kind {other}"),
  }
}

fn line_of(value: &Value) -> Vec<Inline> {
  value.as_array().expect("a line's pieces").iter().map(inline_of).collect()
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

fn document_json(document: &Document) -> Value {
  json!({ "blocks": blocks_json(&document.blocks), "gaps": document.gaps })
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

fn optional(markup: Option<String>) -> Value {
  markup.map_or(Value::Null, Value::from)
}

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "markdown",
    &[
      ("parseInline", &|args| line_json(&parse_inline(&chars(text(&args[0])), None))),
      ("parseMarkdown", &|args| document_json(&parse_markdown(text(&args[0])))),
      ("renderInline", &|args| Value::from(render_inline(&line_of(&args[0])))),
      ("plainInline", &|args| Value::from(plain_inline(&line_of(&args[0])))),
      ("convertMarkdown", &|args| {
        let converted = convert_markdown(text(&args[0]));

        json!({ "markup": converted.markup, "plain": converted.plain, "formatted": converted.formatted })
      }),
      ("foldBlocks", &|args| {
        fold_blocks(&parse_markdown(text(&args[0]))).as_ref().map_or(Value::Null, fold_json)
      }),
      ("foldedMarkup", &|args| optional(folded_markup(&convert_markdown(text(&args[0]))))),
      ("codeMarkup", &|args| optional(code_markup(&convert_markdown(text(&args[0]))))),
      ("markdownParts", &|args| {
        let limit = args
          .get(1)
          .map_or(TEXT_LIMIT, |limit| usize::try_from(limit.as_u64().expect("a limit")).expect("a limit"));

        Value::from(markdown_parts(text(&args[0]), limit))
      }),
    ],
  );
}

#[test]
fn a_text_past_the_limit_is_divided_with_its_fence_closed_and_opened_again() {
  let code = (0..40).map(|at| format!("let value_{at} = {at};")).collect::<Vec<_>>().join("\n");
  let parts = markdown_parts(&format!("Before.\n\n```rust\n{code}\n```"), 300);

  assert!(parts.len() > 2);
  assert!(parts.iter().all(|part| sent_length(&chars(part)) <= 300));
  assert!(parts[1].starts_with("```rust\n"));
  assert!(parts[1].ends_with("\n```"));
}
