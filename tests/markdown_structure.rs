mod shared;

use bmo_text::markdown_structure::*;
use serde_json::{Value, json};

use shared::text;

fn heading(heading: &MarkdownHeading) -> Value {
  json!({ "depth": heading.depth, "text": heading.text, "slug": heading.slug, "line": heading.line })
}

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "markdown-structure",
    &[
      ("markdownHeadings", &|args| {
        markdown_headings(text(&args[0])).iter().map(heading).collect()
      }),
      ("plainHeading", &|args| Value::from(plain_heading(text(&args[0])))),
      ("headingSlug", &|args| Value::from(heading_slug(text(&args[0])))),
      ("markdownSections", &|args| Value::from(markdown_sections(text(&args[0])))),
      ("markdownStructure", &|args| {
        let structure = markdown_structure(text(&args[0]));

        json!({
          "headings": structure.headings.iter().map(heading).collect::<Vec<_>>(),
          "sections": structure.sections,
        })
      }),
      ("halveText", &|args| {
        halve_text(text(&args[0]))
          .map_or(Value::Null, |cut| json!({ "halves": [cut.halves.0, cut.halves.1], "joint": cut.joint }))
      }),
    ],
  );
}
