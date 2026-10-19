use bmo_markup::english::*;

// Two policies of the shapes parts declare: a strict one reading two names past and allowing no Hangul
// left, and a lenient one also reading a phrase past and weighing each syllable against twenty Latin
// letters. The names are Hangul, since what the test reads past is Hangul a part allows.
const STRICT_SET_ASIDE: &str = "가나|다라";
const LENIENT_SET_ASIDE: &str = "(가나|다라) 마바|가나|다라|사아";

// A Hangul syllable no set-aside word holds.
const SYLLABLE: char = '가';

#[test]
fn a_strict_policy_refuses_a_single_hangul_syllable() {
  let test = EnglishTest::new(STRICT_SET_ASIDE, 0);

  assert!(!test.passes(&format!("fix: the {SYLLABLE} reading")));
  assert!(!test.passes(&format!("Land the close\n\nThe body holds one {SYLLABLE}")));
  assert!(!test.passes(&SYLLABLE.to_string()));
}

#[test]
fn a_strict_policy_reads_its_set_aside_names_past_and_passes_pure_english() {
  let test = EnglishTest::new(STRICT_SET_ASIDE, 0);

  for name in STRICT_SET_ASIDE.split('|') {
    assert!(test.passes(&format!("Refuse Hangul in commits, as {name} asked")), "{name}");
    assert!(!test.passes(&format!("Refuse Hangul in commits, as {name}{SYLLABLE} asked")), "{name}");
  }

  assert!(test.passes("docs: state that commits are English only"));
  assert!(test.passes(""));
}

#[test]
fn a_lenient_policy_reads_a_few_syllables_past_against_the_latin_letters() {
  let test = EnglishTest::new(LENIENT_SET_ASIDE, 20);
  let few: String = std::iter::repeat_n(SYLLABLE, 5).collect();

  assert!(test.passes(&format!("The suite rebuilds the server inside its guard, and {SYLLABLE} word")));
  assert!(!test.passes(&format!("{few} with a few words")));
  assert!(!test.passes(&SYLLABLE.to_string()));
}

#[test]
fn the_default_reads_nothing_past_and_refuses_any_hangul() {
  let test = EnglishTest::default();

  assert!(test.passes("plain English"));
  assert!(!test.passes(&format!("almost English {SYLLABLE}")));
}
