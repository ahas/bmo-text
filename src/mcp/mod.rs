//! The MCP server `bmo-text mcp` starts (docs/design.md, 「The MCP server」), compiled only with the `mcp`
//! feature. Each tool takes its input, calls one public function of the crate and writes that function's answer
//! as its structured output, in the units' wire format: camelCase keys, an optional input left out or `null`
//! alike. No reading is made here.

use std::io;
use std::path::PathBuf;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, ServiceExt, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

use crate::answer_language::text_language;
use crate::dashes::read_dashes;
use crate::html_reading::read_html_page;
use crate::markdown_structure::{MarkdownHeading, markdown_structure};
use crate::translation_parts::{TRANSLATION_PART_TOKENS, translation_parts};
use crate::web_address::web_addresses_in;
use crate::wording::text_swaps;

/// The version the server names itself at: the set's version the build was made at, which the testbed
/// hands the build as `BMO_VERSION`, else the manifest's placeholder (bmo-chat's `docs/conventions.md`,
/// 「Versions」).
pub const VERSION: &str = match option_env!("BMO_VERSION") {
  Some(version) => version,
  None => env!("CARGO_PKG_VERSION"),
};

/// What a harness is told of the server when it connects.
const INSTRUCTIONS: &str = "Readings of markdown and plain text, each a pure computation over the text given: \
the dashes left between words, the language a text is written in, a markdown text's headings and sections, an \
HTML page's words a block to a line, the parts a translator is given, the web addresses a text names, and the \
terms a glossary bans. Call them on text before sending or filing it.";

/// The stem the pictures a page carries inside itself are named after in its lines.
const PICTURE_STEM: &str = "picture";

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct DashesInput {
  /// The text to read.
  text: String,
  /// Whether to give the text back with its dashes replaced; false by default, which gives it as it came.
  replace: Option<bool>,
  /// Whether the text is a document wrapped at a fixed width, read a paragraph or a list item at a time across
  /// its line breaks; false by default, which reads each line alone as a chat answer is written.
  wrapped: Option<bool>,
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct DashesOutput {
  /// The text with its dashes replaced where `replace` was asked for, and as given otherwise.
  text: String,
  /// How many dashes between words the text holds; the dashes of code, quotations, addresses and tables are
  /// not counted.
  found: usize,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct TextInput {
  /// The text to read.
  text: String,
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct LanguageOutput {
  /// `en` where the text's letters outside code are mostly Latin, and `ko` otherwise.
  language: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct MarkdownInput {
  /// The markdown text to read.
  markdown: String,
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct HeadingOutput {
  /// 1 for `#`, 6 for `######`.
  depth: usize,
  /// What the heading says, its inline markup taken off.
  text: String,
  /// A mark no other heading of the text carries, made from its words.
  slug: String,
  /// The line the heading stands on, counted from one.
  line: usize,
}

impl From<MarkdownHeading> for HeadingOutput {
  fn from(heading: MarkdownHeading) -> Self {
    Self {
      depth: heading.depth,
      text: heading.text,
      slug: heading.slug,
      line: heading.line,
    }
  }
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct StructureOutput {
  /// Every ATX heading outside fenced code, in order.
  headings: Vec<HeadingOutput>,
  /// The text cut at each heading, the lines before the first kept; joined back, they are the text.
  sections: Vec<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct HtmlInput {
  /// The HTML page to read.
  html: String,
  /// The index of the first line asked for, counted from nought; nought by default.
  from: Option<usize>,
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct HtmlOutput {
  /// The page's title, or null where it has none.
  title: Option<String>,
  /// The page's words, a block to a line, as many as fit under the answer ceiling.
  lines: Vec<String>,
  /// Whether lines are left after these: ask again from `from` plus the number of lines given.
  more: bool,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PartsInput {
  /// The text to cut.
  text: String,
  /// How many estimated tokens one part carries at most; 750 by default.
  limit: Option<usize>,
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PartsOutput {
  /// The parts, in order; joined back, they are the text.
  parts: Vec<String>,
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct AddressesOutput {
  /// Every http or https address the text names, as written and in order; a markdown link gives the address
  /// in its parentheses.
  addresses: Vec<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct WordingInput {
  /// The text to check.
  text: String,
  /// The folder holding the glossary document, `WORDING.md`; the server's working directory by default.
  folder: Option<String>,
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct SwapOutput {
  /// The line the term is used on, counted from one.
  line: usize,
  /// That line as written.
  text: String,
  /// The banned term.
  term: String,
  /// What to write instead.
  instead: String,
  /// When the entry applies, or null where it always does.
  when: Option<String>,
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct WordingOutput {
  /// Every use of a banned term outside the text's quoted spans, in order.
  swaps: Vec<SwapOutput>,
}

/// The server: its seven tools.
#[derive(Clone)]
pub struct TextTools {
  tool_router: ToolRouter<Self>,
}

impl Default for TextTools {
  fn default() -> Self {
    Self {
      tool_router: Self::tool_router(),
    }
  }
}

#[tool_router]
impl TextTools {
  #[tool(
    name = "text_dashes",
    description = "Find the dashes left between words in a text, or replace them the way a person types the \
                   sentence: a colon, commas around an aside, a range between digits."
  )]
  fn dashes(&self, Parameters(input): Parameters<DashesInput>) -> Json<DashesOutput> {
    let read = read_dashes(&input.text, input.wrapped.unwrap_or(false));

    Json(DashesOutput {
      text: if input.replace.unwrap_or(false) { read.text } else { input.text },
      found: read.found,
    })
  }

  #[tool(
    name = "text_language",
    description = "The language a text is written in, ko or en, by its letters outside code."
  )]
  fn language(&self, Parameters(input): Parameters<TextInput>) -> Json<LanguageOutput> {
    Json(LanguageOutput {
      language: text_language(&input.text).code().to_owned(),
    })
  }

  #[tool(
    name = "text_markdown_structure",
    description = "A markdown text's headings outside fenced code, and the sections it is cut into at each \
                   heading."
  )]
  fn markdown_structure(&self, Parameters(input): Parameters<MarkdownInput>) -> Json<StructureOutput> {
    let structure = markdown_structure(&input.markdown);

    Json(StructureOutput {
      headings: structure.headings.into_iter().map(HeadingOutput::from).collect(),
      sections: structure.sections,
    })
  }

  #[tool(
    name = "text_html_reading",
    description = "An HTML page's title and words, a block to a line with headings and list items in markdown, \
                   scripts and styles left out; a long page is read a part at a time from a line index."
  )]
  fn html_reading(&self, Parameters(input): Parameters<HtmlInput>) -> Json<HtmlOutput> {
    let page = read_html_page(&input.html, PICTURE_STEM, input.from.unwrap_or(0));

    Json(HtmlOutput {
      title: page.title,
      lines: page.lines,
      more: page.more,
    })
  }

  #[tool(
    name = "text_translation_parts",
    description = "Split a text into the parts a translation takes, each under a limit of estimated tokens: \
                   its sections packed together while they fit, a section too large cut at its paragraphs."
  )]
  fn translation_parts(&self, Parameters(input): Parameters<PartsInput>) -> Json<PartsOutput> {
    Json(PartsOutput {
      parts: translation_parts(&input.text, input.limit.unwrap_or(TRANSLATION_PART_TOKENS)),
    })
  }

  #[tool(
    name = "text_web_addresses",
    description = "The http and https addresses a text names, bare or in markdown links, each as written with \
                   the sentence's punctuation left off."
  )]
  fn web_addresses(&self, Parameters(input): Parameters<TextInput>) -> Json<AddressesOutput> {
    Json(AddressesOutput {
      addresses: web_addresses_in(&input.text).into_iter().map(|found| found.url).collect(),
    })
  }

  #[tool(
    name = "text_wording",
    description = "Check a text against the glossary of banned terms in a folder's WORDING.md: each use outside \
                   quoted spans, with its line and what to write instead."
  )]
  fn wording(&self, Parameters(input): Parameters<WordingInput>) -> Result<Json<WordingOutput>, String> {
    let folder = match input.folder {
      Some(folder) => PathBuf::from(folder),
      None => std::env::current_dir()
        .map_err(|error| format!("The working directory cannot be read: {error}."))?,
    };
    let swaps = text_swaps(&input.text, &folder)?
      .into_iter()
      .map(|hit| SwapOutput {
        line: hit.line,
        text: hit.text,
        term: hit.term.term,
        instead: hit.term.instead,
        when: hit.term.when,
      })
      .collect();

    Ok(Json(WordingOutput { swaps }))
  }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for TextTools {
  fn get_info(&self) -> ServerConfig {
    ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
      .with_server_info(Implementation::new("bmo-text", VERSION))
      .with_instructions(INSTRUCTIONS)
  }
}

/// Answers MCP over the process's standard input and output until the other side closes them.
pub async fn serve_stdio() -> io::Result<()> {
  let running = TextTools::default().serve(rmcp::transport::stdio()).await.map_err(io::Error::other)?;

  running.waiting().await.map_err(io::Error::other)?;
  Ok(())
}
