use bmo_markup::format::*;

#[test]
fn sound_markup_has_no_fault() {
  for markup in [
    "plain &amp; simple",
    "<b>bold <i>both</i></b>",
    r#"<pre><code class="language-rust">x &lt; y</code></pre>"#,
    r#"<a href="https://example.com">a</a>"#,
    r#"<span class="tg-spoiler">hidden</span>"#,
    "<blockquote expandable>folded</blockquote>",
  ] {
    assert_eq!(markup_fault(markup), None, "{markup}");
  }
}

#[test]
fn each_fault_is_found_first() {
  assert_eq!(markup_fault("a < b"), Some(MarkupFault::Unescaped { character: '<' }));
  assert_eq!(markup_fault("a & b"), Some(MarkupFault::Unescaped { character: '&' }));
  assert_eq!(markup_fault("a > b"), Some(MarkupFault::Unescaped { character: '>' }));
  assert_eq!(markup_fault("<div>x</div>"), Some(MarkupFault::Unsupported { tag: "div".into() }));
  assert_eq!(markup_fault("<span>x</span>"), Some(MarkupFault::Unsupported { tag: "span".into() }));
  assert_eq!(markup_fault("<b>x"), Some(MarkupFault::Unclosed { tag: "b".into() }));
  assert_eq!(
    markup_fault("x</b>"),
    Some(MarkupFault::Unopened {
      tag: "b".into(),
      open: None
    })
  );
  assert_eq!(
    markup_fault("<b><i>x</b></i>"),
    Some(MarkupFault::Unopened {
      tag: "b".into(),
      open: Some("i".into())
    })
  );
  assert_eq!(
    markup_fault("<b><code>x</code></b>"),
    Some(MarkupFault::Nested {
      tag: "code".into(),
      inside: "b".into()
    })
  );
  assert_eq!(
    markup_fault("<blockquote><blockquote>x</blockquote></blockquote>"),
    Some(MarkupFault::Nested {
      tag: "blockquote".into(),
      inside: "blockquote".into()
    })
  );
}

#[test]
fn the_visible_text_is_the_words_alone() {
  assert_eq!(visible_text("<b>a &lt;b&gt; &amp;</b>"), "a <b> &");
}
