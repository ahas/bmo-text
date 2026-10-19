use bmo_markup::js::chars;
use bmo_markup::web_address::*;

#[test]
fn an_address_ends_where_the_sentence_takes_over() {
  assert_eq!(address_as_written("https://example.com/a)."), "https://example.com/a");
  assert_eq!(
    address_as_written("https://en.wikipedia.org/wiki/A_(b))"),
    "https://en.wikipedia.org/wiki/A_(b)"
  );
  assert_eq!(address_as_written("https://example.com/주소"), "https://example.com/");
}

#[test]
fn a_link_and_a_bare_address_are_found_in_order() {
  let found = web_addresses_in("see [docs](https://a.example/x) and https://b.example/y.");

  assert_eq!(
    found.iter().map(|one| one.url.as_str()).collect::<Vec<_>>(),
    ["https://a.example/x", "https://b.example/y"]
  );
  assert_eq!(found[0].start, 11);
}

#[test]
fn a_link_needs_its_parentheses_and_no_space() {
  assert!(markdown_link_at(&chars("[a](b c)"), 0).is_none());
  assert!(markdown_link_at(&chars("[a] (b)"), 0).is_none());
  assert_eq!(markdown_link_at(&chars("[a [b]](c(d))"), 0).map(|link| link.url), Some("c(d)".into()));
}
