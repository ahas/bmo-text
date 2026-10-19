mod shared;

use bmo_text::format::*;
use serde_json::{Value, json};

use shared::text;

// A fault in the package's shape, a tagged union by its kind.
fn fault_json(fault: &MarkupFault) -> Value {
  match fault {
    MarkupFault::Unescaped { character } => {
      json!({ "kind": "unescaped", "character": character.to_string() })
    }
    MarkupFault::Unsupported { tag } => json!({ "kind": "unsupported", "tag": tag }),
    MarkupFault::Unclosed { tag } => json!({ "kind": "unclosed", "tag": tag }),
    MarkupFault::Unopened { tag, open } => json!({ "kind": "unopened", "tag": tag, "open": open }),
    MarkupFault::Nested { tag, inside } => json!({ "kind": "nested", "tag": tag, "inside": inside }),
  }
}

// A fault read back from the package's shape.
fn fault_of(value: &Value) -> MarkupFault {
  let field = |key: &str| text(&value[key]).to_owned();

  match text(&value["kind"]) {
    "unescaped" => MarkupFault::Unescaped {
      character: text(&value["character"]).chars().next().expect("a character"),
    },
    "unsupported" => MarkupFault::Unsupported { tag: field("tag") },
    "unclosed" => MarkupFault::Unclosed { tag: field("tag") },
    "unopened" => MarkupFault::Unopened {
      tag: field("tag"),
      open: value["open"].as_str().map(str::to_owned),
    },
    "nested" => MarkupFault::Nested {
      tag: field("tag"),
      inside: field("inside"),
    },
    other => panic!("no fault of kind {other}"),
  }
}

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "format",
    &[
      ("escapeHtml", &|args| Value::from(escape_html(text(&args[0])))),
      ("escapeAttribute", &|args| Value::from(escape_attribute(text(&args[0])))),
      ("isLinkable", &|args| Value::from(is_linkable(text(&args[0])))),
      ("visibleText", &|args| Value::from(visible_text(text(&args[0])))),
      ("expandableMarkup", &|args| Value::from(expandable_markup(text(&args[0])))),
      ("expandable", &|args| Value::from(expandable(text(&args[0])))),
      ("markupFault", &|args| {
        markup_fault(text(&args[0])).as_ref().map_or(Value::Null, fault_json)
      }),
      ("faultReason", &|args| Value::from(fault_reason(&fault_of(&args[0])))),
    ],
  );
}
