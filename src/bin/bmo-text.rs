//! The crate's command. `bmo-text mcp` starts the MCP server over standard input and output, for the harness
//! that runs it (docs/design.md, 「The MCP server」).

use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Readings of markdown and text.
#[derive(Parser)]
#[command(name = "bmo-text", version = bmo_text::mcp::VERSION)]
struct Command {
  #[command(subcommand)]
  verb: Verb,
}

#[derive(Subcommand)]
enum Verb {
  /// Serve the text tools over MCP on standard input and output until the other side closes them.
  Mcp,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
  match Command::parse().verb {
    Verb::Mcp => match bmo_text::mcp::serve_stdio().await {
      Ok(()) => ExitCode::SUCCESS,
      Err(error) => {
        eprintln!("bmo-text mcp: {error}");
        ExitCode::FAILURE
      }
    },
  }
}
