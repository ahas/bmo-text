use bmo_markup::wording::*;

#[test]
fn a_banned_word_is_found_outside_quotes_where_it_stands_alone_and_not_in_its_own_sense() {
  let glossary = parse_glossary(
    "# Wording\n\n## Banned terms\n\n- ko · 다리 → 경로\n- en · utilise → use — always\n\n## Uses in the word's own sense\n\n- ko · 다리 · 다리 밑 — under a bridge\n",
  )
  .unwrap();
  let found =
    |text: &str| banned_terms_in(text, &glossary).into_iter().map(|term| term.term).collect::<Vec<_>>();

  assert_eq!(found("이 다리로 가"), vec!["다리"]);
  assert!(found("`다리`라고 쓰지 마").is_empty());
  assert!(found("다리 밑에 있어").is_empty());
  assert!(found("사다리를 올라").is_empty());
  assert_eq!(found("we utilise it"), vec!["utilise"]);
  assert_eq!(banned_terms_line(&banned_terms_in("이 다리", &glossary)), "\"다리\" → 경로");
  assert!(parse_glossary("## Banned terms\n- xx · a → b\n").is_err());
}

const SAMPLE: &str = "# Wording\n\n## Banned terms\n\n- Each line is `<language code> · <banned term> → <what to write>`.\n- en · tool strip → toolbar\n- en · fixing → editing — when it means editing\n- en · slot → position\n\n## Uses in the word's own sense\n\n- Each line is `<language code> · <banned term> · <a phrase holding the term>`.\n- en · slot · slot machine — a machine\n- en · slot · coin slot — an opening\n\n## Allowed uses\n\n- en · fixing · docs/a.md · bike fixing — a repair\n";

fn term(term: &str, instead: &str, when: Option<&str>) -> BannedTerm {
  BannedTerm {
    lang: "en".to_owned(),
    term: term.to_owned(),
    instead: instead.to_owned(),
    when: when.map(str::to_owned),
    keeps: vec![],
  }
}

#[test]
fn the_lists_are_read_from_their_own_sections_and_the_explanation_bullets_are_not() {
  let glossary = parse_wording(SAMPLE).unwrap();
  let mut slot = term("slot", "position", None);

  slot.keeps = vec!["slot machine".to_owned(), "coin slot".to_owned()];

  assert_eq!(
    glossary.banned,
    [
      term("tool strip", "toolbar", None),
      term("fixing", "editing", Some("when it means editing")),
      slot
    ]
  );
  assert_eq!(
    glossary.allowed,
    [AllowedUse {
      lang: "en".to_owned(),
      term: "fixing".to_owned(),
      path: "docs/a.md".to_owned(),
      excerpt: "bike fixing".to_owned(),
    }]
  );
}

#[test]
fn new_banned_lines_go_after_the_last_entry_and_read_back_as_written() {
  let saving = term("stashing", "saving, capturing", Some("when it means saving"));
  let (text, added) = add_banned_terms(SAMPLE, &[saving.clone(), term("slot", "position", None)]).unwrap();
  let lines: Vec<&str> = text.split('\n').collect();
  let after = lines.iter().position(|line| *line == "- en · slot → position").unwrap() + 1;

  assert_eq!(added, std::slice::from_ref(&saving));
  assert_eq!(lines[after], "- en · stashing → saving, capturing — when it means saving");
  assert_eq!(parse_glossary(&text).unwrap().last(), Some(&saving));
  assert_eq!(add_banned_terms(SAMPLE, &[term("slot", "position", None)]).unwrap().0, SAMPLE);
  assert!(
    add_banned_terms(SAMPLE, &[term("a → b", "c", None)])
      .unwrap_err()
      .contains("would not read back")
  );
}

#[test]
fn a_term_is_found_a_line_at_a_time_on_lines_of_its_own_language_and_not_in_its_own_sense() {
  let banned = parse_glossary(SAMPLE).unwrap();
  // A Korean word makes its line a Korean one.
  let korean_line = "위에 tool strip";
  let hits =
    find_banned(&format!("{korean_line}\nthe tool strip above\na slot machine\nan empty slot"), &banned);

  assert_eq!(line_language(korean_line), "ko");
  assert_eq!(line_language("\u{30c4}\u{30fc}\u{30eb}"), "ja");
  assert_eq!(
    hits.iter().map(|hit| (hit.line, hit.term.term.as_str())).collect::<Vec<_>>(),
    [(2, "tool strip"), (4, "slot")]
  );
  assert_eq!(hits[0].text, "the tool strip above");
}
