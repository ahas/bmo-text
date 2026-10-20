//! The repository's own shape, held for the crate at the root and the package beside it: the crate's
//! manifest the only one at the root, no directory named after its parent, tests only under a `tests/`
//! folder, no dependency on another bmo repository, one version, and Korean only where the ledger allows
//! it and never hidden behind an escape (docs/design.md).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

/// The repository root, where the crate stands.
fn root() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).to_owned()
}

/// Every file of the checkout, tracked or new and not ignored, relative to the root.
fn checkout_files() -> Vec<String> {
  let output = Command::new("git")
    .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
    .current_dir(root())
    .output()
    .expect("git ls-files runs");

  assert!(
    output.status.success(),
    "git ls-files failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  String::from_utf8(output.stdout)
    .expect("paths are UTF-8")
    .split('\0')
    .filter(|path| !path.is_empty() && root().join(path).is_file())
    .map(str::to_owned)
    .collect()
}

fn text_of(path: &str) -> String {
  fs::read_to_string(root().join(path)).unwrap_or_default()
}

/// File names that declare a package or a project to a build tool.
const MANIFESTS: [&str; 9] = [
  "Cargo.toml",
  "package.json",
  "pyproject.toml",
  "setup.py",
  "go.mod",
  "deno.json",
  "pnpm-workspace.yaml",
  "pom.xml",
  "build.gradle",
];

/// The extensions of a code file, the files the Korean ledger counts.
const CODE: [&str; 10] = ["rs", "ts", "tsx", "js", "mjs", "cjs", "vue", "py", "sh", "css"];

fn is_code(path: &str) -> bool {
  path.rsplit_once('.').is_some_and(|(_, extension)| CODE.contains(&extension))
}

/// The Hangul blocks: jamo, compatibility jamo, extended jamo A, the syllables and extended jamo B.
fn is_hangul(code: u32) -> bool {
  matches!(code, 0x1100..=0x11ff | 0x3130..=0x318f | 0xa960..=0xa97f | 0xac00..=0xd7af | 0xd7b0..=0xd7ff)
}

fn hangul_lines(text: &str) -> usize {
  text
    .lines()
    .filter(|line| line.chars().any(|character| is_hangul(character.into())))
    .count()
}

/// The one manifest the root holds: the crate's.
const ROOT_MANIFEST: &str = "Cargo.toml";

#[test]
fn only_the_crate_s_manifest_stands_at_the_root() {
  let found: Vec<String> = checkout_files()
    .into_iter()
    .filter(|path| MANIFESTS.contains(&path.as_str()) && path != ROOT_MANIFEST)
    .collect();

  assert!(
    found.is_empty(),
    "the root holds {found:?}; only the crate's manifest stands there, the package keeps its own"
  );
}

#[test]
fn no_directory_is_named_after_its_parent() {
  let twins: Vec<String> = checkout_files()
    .into_iter()
    .filter(|path| {
      let folders: Vec<&str> = path.split('/').collect();

      folders[..folders.len() - 1].windows(2).any(|pair| pair[0] == pair[1])
    })
    .collect();

  assert!(twins.is_empty(), "these paths repeat a folder's name inside itself: {twins:?}");
}

/// Whether a Rust file holds a test module or a test function.
fn holds_rust_test(text: &str) -> bool {
  text.lines().any(|line| matches!(line.trim(), "#[cfg(test)]" | "#[test]"))
}

#[test]
fn tests_live_only_in_tests() {
  let misplaced: Vec<String> = checkout_files()
    .into_iter()
    .filter(|path| {
      let in_tests = path.starts_with("tests/") || path.starts_with("package/tests/");
      let test_file = path.contains(".test.") || path.contains(".spec.");
      let rust_test = path.ends_with(".rs") && holds_rust_test(&text_of(path));

      !in_tests && (test_file || rust_test)
    })
    .collect();

  assert!(misplaced.is_empty(), "these tests stand outside a tests folder: {misplaced:?}");
}

fn is_bmo(name: &str) -> bool {
  name.starts_with("bmo-") || name.starts_with("bmo_") || name.starts_with("@bmo/")
}

/// Every dependency a Cargo manifest names, in every dependency table, a target's included.
fn crate_dependencies(manifest: &toml::Table) -> Vec<String> {
  let tables = ["dependencies", "dev-dependencies", "build-dependencies"];
  let names = |table: &toml::Table| -> Vec<String> {
    tables
      .iter()
      .filter_map(|key| table.get(*key)?.as_table())
      .flat_map(|listed| listed.keys().cloned())
      .collect()
  };
  let targets = manifest.get("target").and_then(toml::Value::as_table);

  names(manifest)
    .into_iter()
    .chain(
      targets
        .into_iter()
        .flat_map(|targets| targets.values())
        .filter_map(toml::Value::as_table)
        .flat_map(names),
    )
    .collect()
}

/// Every dependency a package manifest names, in every dependency list.
fn package_dependencies(manifest: &Value) -> Vec<String> {
  ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"]
    .iter()
    .filter_map(|key| manifest.get(*key)?.as_object())
    .flat_map(|listed| listed.keys().cloned())
    .collect()
}

fn cargo_manifest() -> toml::Table {
  text_of(ROOT_MANIFEST).parse().expect("the crate's manifest is TOML")
}

fn package_manifest() -> Value {
  serde_json::from_str(&text_of("package/package.json")).expect("the package's manifest is JSON")
}

#[test]
fn neither_manifest_depends_on_another_bmo_repository() {
  let crates: Vec<String> =
    crate_dependencies(&cargo_manifest()).into_iter().filter(|name| is_bmo(name)).collect();
  let packages: Vec<String> = package_dependencies(&package_manifest())
    .into_iter()
    .filter(|name| is_bmo(name))
    .collect();

  assert!(
    crates.is_empty(),
    "the crate depends on {crates:?}; a base library stands on registry crates alone"
  );
  assert!(
    packages.is_empty(),
    "the package depends on {packages:?}; a base library stands on registry packages alone"
  );
}

#[test]
fn the_crate_and_the_package_carry_the_repository_version() {
  let version = text_of("VERSION").trim().to_owned();
  let crate_version = cargo_manifest()["package"]["version"].as_str().map(str::to_owned);
  let package_version = package_manifest()["version"].as_str().map(str::to_owned);

  assert!(!version.is_empty(), "VERSION names the repository's version");
  assert_eq!(crate_version.as_deref(), Some(version.as_str()), "the crate's version is VERSION's");
  assert_eq!(package_version.as_deref(), Some(version.as_str()), "the package's version is VERSION's");
  assert_eq!(
    plugin_manifest()["version"].as_str(),
    Some(version.as_str()),
    "the plugin's version is VERSION's"
  );
}

fn plugin_manifest() -> Value {
  serde_json::from_str(&text_of("plugin/.claude-plugin/plugin.json"))
    .expect("the plugin's manifest is JSON")
}

#[test]
fn the_plugin_starts_the_crate_s_mcp_server_and_carries_its_skill() {
  let servers: Value =
    serde_json::from_str(&text_of("plugin/.mcp.json")).expect("the plugin's MCP configuration is JSON");
  let skill = text_of("plugin/skills/ahas-text/SKILL.md");

  assert_eq!(plugin_manifest()["name"], "ahas-text");
  assert_eq!(servers["mcpServers"]["bmo-text"]["command"], "bmo-text");
  assert_eq!(servers["mcpServers"]["bmo-text"]["args"], serde_json::json!(["mcp"]));
  assert!(skill.starts_with("---\nname: \"ahas-text\"\n"), "the skill names itself ahas-text");
}

// The names of the files a folder holds with an extension, the extension taken off and `_` read as `-`.
fn module_names(folder: &str, extension: &str) -> Vec<String> {
  let mut names: Vec<String> = fs::read_dir(root().join(folder))
    .expect("the folder reads")
    .map(|entry| entry.expect("an entry").path())
    .filter(|path| path.extension().is_some_and(|found| found == extension))
    .map(|path| path.file_stem().expect("a name").to_string_lossy().replace('_', "-"))
    .collect();

  names.sort();
  names
}

#[test]
fn the_crate_and_the_package_carry_one_list_of_modules_each_held_to_its_cases() {
  // The crate's `js` is how it reads as JavaScript does, which the package has from its language.
  let crate_modules: Vec<String> = module_names("src", "rs")
    .into_iter()
    .filter(|name| name != "lib" && name != "js")
    .collect();
  let mut package_modules: Vec<String> = package_manifest()["exports"]
    .as_object()
    .expect("the package's exports")
    .keys()
    .map(|key| key.trim_start_matches("./").to_owned())
    .collect();

  package_modules.sort();

  assert_eq!(crate_modules, package_modules, "the crate's modules are the package's");
  assert_eq!(module_names("cases", "json"), package_modules, "each module is held to one cases file");
}

// An escape naming a code point, in either brace or four-digit form or a regular expression's `\x{...}`, and
// a call building a character from a number.
static ESCAPE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\\[ux]\{([0-9A-Fa-f]{1,6})\}|\\u([0-9A-Fa-f]{4})").expect("an escape"));
static CHAR_CODE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"(?:fromCharCode|fromCodePoint|from_u32|from_u32_unchecked)\(\s*(0x[0-9A-Fa-f_]+|[0-9_]+)")
    .expect("a character built from a number")
});
// The UTF-8 bytes of a Hangul character written as byte escapes: a lead byte from E1 to ED and two more.
static HANGUL_BYTES: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"\\x(?:[eE][1-9a-dA-D])\\x[89abAB][0-9A-Fa-f]\\x[89abAB][0-9A-Fa-f]").expect("Hangul bytes")
});

fn number(written: &str) -> Option<u32> {
  let digits = written.replace('_', "");

  match digits.strip_prefix("0x") {
    Some(hex) => u32::from_str_radix(hex, 16).ok(),
    None => digits.parse().ok(),
  }
}

/// The lines of a text holding Korean written as an escape or a character code.
fn hidden_hangul(text: &str) -> Vec<String> {
  text
    .lines()
    .filter(|line| {
      let escaped = ESCAPE.captures_iter(line).any(|found| {
        let code = found.get(1).or_else(|| found.get(2)).map(|code| code.as_str());

        code.and_then(|code| u32::from_str_radix(code, 16).ok()).is_some_and(is_hangul)
      });
      let built = CHAR_CODE.captures_iter(line).any(|found| number(&found[1]).is_some_and(is_hangul));

      escaped || built || HANGUL_BYTES.is_match(line)
    })
    .map(|line| line.trim().to_owned())
    .collect()
}

#[test]
fn no_code_file_hides_korean_behind_an_escape() {
  let hidden: Vec<String> = checkout_files()
    .into_iter()
    .filter(|path| is_code(path))
    .flat_map(|path| hidden_hangul(&text_of(&path)).into_iter().map(move |line| format!("{path}: {line}")))
    .collect();

  assert!(
    hidden.is_empty(),
    "Korean is written plainly, never as an escape or a char code: {hidden:#?}"
  );
}

#[test]
fn the_escape_check_finds_each_form() {
  // The first Hangul syllable, put together here so that this file does not itself hold the forms it
  // looks for.
  let (hex, decimal, slash) = ("AC00", u32::from_str_radix("AC00", 16).unwrap(), "\\");
  let syllable = [
    format!("{slash}u{{{hex}}}"),
    format!("{slash}u{hex}"),
    format!("{slash}x{{{}}}", hex.to_lowercase()),
    format!("fromCharCode(0x{hex})"),
    format!("from_u32({decimal})"),
    format!("{slash}xea{slash}xb0{slash}x80"),
  ];

  for written in syllable {
    assert_eq!(hidden_hangul(&format!("let a = \"{written}\";")).len(), 1, "{written}");
  }

  for written in ["\\u{2014}", "\\u3000", "\\x{a0}", "fromCharCode(65)", "0xa076_1d64_78bd_642f"] {
    assert!(hidden_hangul(&format!("let a = \"{written}\";")).is_empty(), "{written}");
  }
}

fn ledger() -> BTreeMap<String, usize> {
  serde_json::from_str(&text_of("korean-ledger.json"))
    .expect("the ledger is a JSON object from paths to counts")
}

#[test]
fn every_code_file_holds_no_more_korean_lines_than_the_ledger_allows() {
  let ledger = ledger();
  let over: Vec<String> = checkout_files()
    .into_iter()
    .filter(|path| is_code(path))
    .filter_map(|path| {
      let held = hangul_lines(&text_of(&path));
      let allowed = ledger.get(&path).copied().unwrap_or(0);

      (held > allowed).then(|| format!("{path}: {held} lines, the ledger allows {allowed}"))
    })
    .collect();

  assert!(over.is_empty(), "these files hold more Korean than the ledger allows: {over:#?}");
}

#[test]
fn the_ledger_names_only_code_files_that_stand() {
  let files = checkout_files();
  let stale: Vec<String> =
    ledger().into_keys().filter(|path| !is_code(path) || !files.contains(path)).collect();

  assert!(
    stale.is_empty(),
    "the ledger names files that are not code files of the checkout: {stale:?}"
  );
}
