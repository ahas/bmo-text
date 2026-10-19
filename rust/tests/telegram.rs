use bmo_markup::js::units;
use bmo_markup::telegram::*;

#[test]
fn a_reading_is_folded_under_its_voice_as_a_caption_and_messages_with_nothing_lost() {
  assert_eq!(
    transcript_parts("a <b> & c"),
    vec!["<blockquote expandable>a &lt;b&gt; &amp; c</blockquote>"]
  );
  assert!(transcript_parts("").is_empty());

  let long = "word ".repeat(1_000);
  let parts = transcript_parts(&long);
  let inner = |part: &str| {
    part
      .trim_start_matches("<blockquote expandable>")
      .trim_end_matches("</blockquote>")
      .to_owned()
  };

  assert_eq!(parts.len(), 2);
  assert_eq!(units(&inner(&parts[0])), CAPTION_LIMIT);
  assert_eq!(parts.iter().map(|part| inner(part)).collect::<String>(), long);
}

#[test]
fn words_within_one_message_stand_whole() {
  assert_eq!(plain_parts("short words", 20), vec!["short words"]);
  assert_eq!(plain_parts("", 20), vec![""]);
}

#[test]
fn a_long_text_is_cut_after_the_last_pause_that_fits_keeping_the_pause_with_the_words_before_it() {
  assert_eq!(plain_parts("first part\n\nsecond part", 16), vec!["first part\n\n", "second part"]);
  assert_eq!(plain_parts("one line\nanother line", 12), vec!["one line\n", "another line"]);
  assert_eq!(plain_parts("several words in a row", 12), vec!["several ", "words in a ", "row"]);
}

#[test]
fn a_run_with_no_pause_is_cut_between_whole_characters_counted_as_telegram_counts() {
  // Each of these takes two UTF-16 units, and a cluster of a skin tone takes four.
  assert_eq!(plain_parts("😀😀😀", 5), vec!["😀😀", "😀"]);
  assert_eq!(plain_parts("👍🏽👍🏽", 6), vec!["👍🏽", "👍🏽"]);
  assert_eq!(plain_parts("abcdef", 4), vec!["abcd", "ef"]);
}
