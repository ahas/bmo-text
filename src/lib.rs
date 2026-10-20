//! Markdown and text as a chat room reads them, one twin of the `bmo-text` package: the markdown read
//! line for line into Telegram's HTML and plain text, the structure check every markup is read by, the
//! fold, the division into messages, the dash net under the writing rules, the test of whether a text is
//! English, the net of the terms a glossary bans, the answer ceiling, the structure of a markdown document,
//! the parts a translator is given, a search finding's structure lines and an HTML page read into lines,
//! and the MCP server through which a coding harness calls the pure readings among them.
//! Every length here is counted as the package and Telegram count it, in UTF-16 units, unless its name says
//! `chars`, and every space and trim is JavaScript's, so an input comes out the same on either side.

pub mod answer_cap;
pub mod answer_language;
pub mod dashes;
pub mod english;
pub mod finding_markdown;
pub mod format;
pub mod html_reading;
pub mod js;
pub mod markdown;
pub mod markdown_structure;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod translation_parts;
#[cfg(feature = "wasm")]
#[doc(hidden)]
pub mod wasm;
pub mod web_address;
pub mod wording;
