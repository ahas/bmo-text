mod shared;

use bmo_markup::finding_markdown::*;
use serde_json::Value;

use shared::text;

// The words a case passes, in the package's shape, or the default where it passes none.
fn words(value: Option<&Value>) -> FindingWords {
  let Some(value) = value else {
    return FindingWords::default();
  };
  let word = |key: &str| text(&value[key]).to_owned();

  FindingWords {
    parts: value["parts"]
      .as_object()
      .expect("the parts' words")
      .iter()
      .map(|(key, word)| (key.clone(), text(word).to_owned()))
      .collect(),
    address: word("address"),
    map: word("map"),
    street_view: word("streetView"),
    released: word("released"),
    newer: word("newer"),
    product_page: word("productPage"),
    stock: word("stock"),
    usual: word("usual"),
    deal: word("deal"),
    review: word("review"),
    no_rating: word("noRating"),
    menu: word("menu"),
    dish: word("dish"),
    price: word("price"),
    notes: word("notes"),
  }
}

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "finding-markdown",
    &[
      ("findingMarkdown", &|args| {
        Value::from(finding_markdown(text(&args[0]), &words(args.get(1))))
      }),
      ("footnoteMarks", &|args| Value::from(footnote_marks(text(&args[0])))),
      ("pictureCaption", &|args| Value::from(picture_caption(text(&args[0])))),
      ("findingExcerpt", &|args| {
        let texts: Vec<&str> = args[0].as_array().expect("the findings").iter().map(text).collect();
        let limit = usize::try_from(args[1].as_u64().expect("a limit")).expect("a limit");

        Value::from(finding_excerpt(&texts, limit))
      }),
    ],
  );
}

#[test]
fn a_drawn_gallery_and_map_are_the_blocks_a_renderer_reads() {
  let finding = [
    "NAME: Corner Kitchen",
    "ADDRESS: Riverside",
    "MAP: https://map.example.com/a",
    "IMAGE: https://example.com/a.jpg :: one",
    "IMAGE: https://example.com/b.jpg :: two",
  ];
  let drawn = finding_markdown(&finding.join("\n"), &FindingWords::default());
  let gallery = drawn.find("::: gallery").expect("a gallery is drawn");
  let map = drawn.find("::: map").expect("a map is drawn");

  assert_eq!(&FINDING_BLOCK.captures(&drawn[gallery..]).expect("the gallery reads")[1], "gallery");
  assert_eq!(&FINDING_MAP.captures(&drawn[map..]).expect("the map reads")[1], "Riverside");
}
