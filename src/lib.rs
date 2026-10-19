//! Markdown and text as a chat room reads them, one twin of the `bmo-markup` package: the markdown read
//! line for line into Telegram's HTML and plain text, the structure check every markup is read by, the
//! fold, the division into messages, the dash net under the writing rules, the test of whether a text is
//! English and the net of the terms a glossary bans. Every length here is counted as the package and
//! Telegram count it, in UTF-16 units, and every space and trim is JavaScript's, so a text comes out the
//! same on either side.

pub mod dashes;
pub mod english;
pub mod format;
pub mod js;
pub mod markdown;
pub mod telegram;
pub mod web_address;
pub mod wording;
