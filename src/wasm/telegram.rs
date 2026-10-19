use wasm_bindgen::prelude::*;

use super::{count, number};
use crate::telegram::{
  CAPTION_LIMIT, MAX_SEND_FILE_BYTES, TEXT_LIMIT, bot_id_from_token, plain_parts, transcript_parts,
};

#[wasm_bindgen(js_name = telegramTextLimit)]
pub fn telegram_text_limit() -> f64 {
  number(TEXT_LIMIT)
}

#[wasm_bindgen(js_name = telegramCaptionLimit)]
pub fn telegram_caption_limit() -> f64 {
  number(CAPTION_LIMIT)
}

#[wasm_bindgen(js_name = maxSendFileBytes)]
pub fn max_send_file_bytes() -> f64 {
  number(MAX_SEND_FILE_BYTES)
}

/// The bot's user id as a number, as the package gives it.
#[wasm_bindgen(js_name = botIdFromToken)]
pub fn bot_id_from_token_of(token: &str) -> f64 {
  bot_id_from_token(token) as f64
}

#[wasm_bindgen(js_name = plainParts)]
pub fn plain_parts_of(text: &str, limit: f64) -> Vec<String> {
  plain_parts(text, count(limit))
}

#[wasm_bindgen(js_name = transcriptParts)]
pub fn transcript_parts_of(text: &str) -> Vec<String> {
  transcript_parts(text)
}
