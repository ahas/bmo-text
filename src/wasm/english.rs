use wasm_bindgen::prelude::*;

use super::count;
use crate::english;

/// The crate's `EnglishTest`, held for the package's class of the same name.
#[wasm_bindgen]
pub struct EnglishTest(english::EnglishTest);

#[wasm_bindgen]
impl EnglishTest {
  /// The test `try_new` makes, throwing the compiler's message for a pattern that does not compile.
  #[wasm_bindgen(constructor)]
  pub fn new(set_aside: &str, latin_per_hangul: f64) -> Result<EnglishTest, JsError> {
    english::EnglishTest::try_new(set_aside, count(latin_per_hangul))
      .map(Self)
      .map_err(|error| JsError::new(&error))
  }

  pub fn passes(&self, text: &str) -> bool {
    self.0.passes(text)
  }
}
