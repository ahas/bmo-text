use bmo_markup::js::*;

#[test]
fn bun_s_hash_is_zig_s_wyhash() {
  let cases = [
    ("", 290_873_116_282_709_081),
    ("a", 2_941_419_223_392_617_777),
    ("abc", 190_542_993_387_777_138),
    ("message digest", 4_742_346_353_319_458_430),
    ("abcdefghijklmnopqrstuvwxyz", 8_597_275_765_368_073_426),
    (
      "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
      5_216_907_898_075_982_647,
    ),
    ("한글 문서", 2_141_189_721_873_594_033),
  ];

  for (text, hashed) in cases {
    assert_eq!(bun_hash(text), hashed, "{text}");
  }

  assert_eq!(bun_hash(&"x".repeat(48)), 10_889_483_319_867_276_750);
  assert_eq!(bun_hash(&"y".repeat(49)), 16_226_373_637_292_149_739);
  assert_eq!(bun_hash(&"z".repeat(100)), 10_805_277_514_751_986_052);
}

#[test]
fn white_space_is_javascript_s() {
  assert!(is_space('\u{feff}'));
  assert!(!is_space('\u{85}'));
  assert_eq!(trim("\u{feff} a \u{3000}"), "a");
}

#[test]
fn a_length_is_counted_in_utf16_units() {
  assert_eq!(units("a\u{1f600}"), 3);
  assert_eq!(within(&chars("a\u{1f600}b"), 2), 1);
  assert_eq!(within(&chars("a\u{1f600}b"), 3), 2);
}

#[test]
fn a_run_is_found_from_either_end() {
  let text = chars("a\n\nb\n\n");

  assert_eq!(find(&text, &chars("\n\n"), 0), Some(1));
  assert_eq!(rfind(&text, &chars("\n\n"), 9), Some(4));
  assert_eq!(rfind(&text, &chars("\n\n"), 3), Some(1));
  assert_eq!(find(&text, &chars("x"), 0), None);
}
