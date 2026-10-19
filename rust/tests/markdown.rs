use bmo_markup::js::chars;
use bmo_markup::markdown::*;

fn markup(source: &str) -> String {
  convert_markdown(source).markup
}

#[test]
fn pairs_codes_and_links_become_their_tags() {
  assert_eq!(
    markup("**bold** and *it* and ~~gone~~ and ||hid||"),
    "<b>bold</b> and <i>it</i> and <s>gone</s> and <tg-spoiler>hid</tg-spoiler>"
  );
  assert_eq!(markup("a `x < y` b"), "a <code>x &lt; y</code> b");
  assert_eq!(markup("[site](https://example.com)"), "<a href=\"https://example.com\">site</a>");
  assert_eq!(markup("2**3 and server/**/*.ts"), "2**3 and server/**/*.ts");
}

#[test]
fn a_code_inside_a_bold_splits_the_bold_around_it() {
  assert_eq!(markup("**a `b` c**"), "<b>a </b><code>b</code><b> c</b>");
}

#[test]
fn blocks_are_laid_flat_and_spaced_as_written() {
  let converted = convert_markdown("# Title\n\n- one\n- two\n\n---\n\n> quoted\n\n```rust\nfn x() {}\n```");

  assert_eq!(
    converted.markup,
    "Title\n\n\u{2022} one\n\u{2022} two\n\n<blockquote>quoted</blockquote>\n\n<pre><code class=\"language-rust\">fn x() {}</code></pre>"
  );
  assert_eq!(converted.plain, "Title\n\n\u{2022} one\n\u{2022} two\n\nquoted\n\nfn x() {}");
}

#[test]
fn a_table_is_laid_flat_into_its_cells() {
  assert_eq!(markup("| a | b |\n|---|---|\n| `x|y` | 2 |"), "a \u{b7} b\n<code>x|y</code> \u{b7} 2");
}

#[test]
fn only_the_first_bold_stands_unless_each_is_a_whole_line() {
  assert_eq!(markup("**a** and **b**"), "<b>a</b> and b");
  assert_eq!(markup("- **a**\n- **b**"), "\u{2022} <b>a</b>\n\u{2022} <b>b</b>");
}

#[test]
fn a_long_text_folds_behind_a_quote_with_its_conclusion_open() {
  let body = (0..12).map(|at| format!("line {at} of the body")).collect::<Vec<_>>().join("\n");
  let converted = convert_markdown(&format!("Opening.\n\n{body}\n\nThat is all."));

  assert_eq!(
    code_markup(&converted),
    Some(format!("Opening.\n\n<blockquote expandable>{body}</blockquote>\n\nThat is all."))
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
