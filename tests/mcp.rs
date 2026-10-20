//! The MCP server as a harness meets it: the crate's binary started as `bmo-text mcp`, spoken to over its
//! standard input and output. What each reading answers is its own module's tests'; these hold what only the
//! running server shows: the tools it lists, each tool's structured answer in the wire shape, its error result
//! and its end when the harness closes its input.

#![cfg(feature = "mcp")]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

/// The server, taken down when the test ends however it ends.
struct Server {
  child: Child,
  input: Option<ChildStdin>,
  output: BufReader<ChildStdout>,
  next: u64,
}

impl Server {
  /// The binary started as a harness starts it, in `folder`, past the MCP handshake.
  fn started(folder: &std::path::Path) -> Self {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bmo-text"))
      .arg("mcp")
      .current_dir(folder)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .expect("the server starts");
    let input = child.stdin.take();
    let output = BufReader::new(child.stdout.take().expect("the server's output"));
    let mut server = Self {
      child,
      input,
      output,
      next: 0,
    };
    let info = server.ask(
      "initialize",
      json!({
        "protocolVersion": "2025-06-18",
        "capabilities": {},
        "clientInfo": { "name": "tester", "version": "0.0.0" },
      }),
    );

    assert_eq!(info["serverInfo"]["name"], "bmo-text");
    assert_eq!(info["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));
    server.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
    server
  }

  fn send(&mut self, message: &Value) {
    let input = self.input.as_mut().expect("the server's input is open");

    writeln!(input, "{message}").expect("the server reads");
    input.flush().expect("the server reads");
  }

  /// The result of one request, read past anything else the server says first.
  fn ask(&mut self, method: &str, params: Value) -> Value {
    self.next += 1;
    let id = self.next;

    self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));

    loop {
      let mut line = String::new();

      assert!(self.output.read_line(&mut line).expect("the server answers") > 0, "the server ended");

      let message: Value = serde_json::from_str(&line).expect("the server answers JSON");

      if message["id"] == id {
        assert!(message.get("error").is_none(), "{method} failed: {message}");
        return message["result"].clone();
      }
    }
  }

  fn call(&mut self, tool: &str, arguments: Value) -> Value {
    self.ask("tools/call", json!({ "name": tool, "arguments": arguments }))
  }
}

impl Drop for Server {
  fn drop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

/// A tool's structured answer, which its text content repeats.
fn answered(result: &Value) -> Value {
  assert_eq!(result["isError"], false, "{result}");

  let structured = result["structuredContent"].clone();
  let text: Value = serde_json::from_str(result["content"][0]["text"].as_str().expect("a text content"))
    .expect("the text content is JSON");

  assert_eq!(text, structured);
  structured
}

/// A folder of its own for one test, taken down when the test ends.
struct Folder(std::path::PathBuf);

impl Folder {
  fn new(name: &str) -> Self {
    let folder = std::env::temp_dir().join(format!("bmo-text-mcp-{name}-{}", std::process::id()));

    std::fs::create_dir_all(&folder).expect("the folder is made");
    Self(folder)
  }
}

impl Drop for Folder {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.0);
  }
}

#[test]
fn the_server_lists_exactly_its_seven_tools_each_with_both_schemas() {
  let folder = Folder::new("list");
  let mut server = Server::started(&folder.0);
  let listed = server.ask("tools/list", json!({}));
  let tools = listed["tools"].as_array().expect("a list of tools");
  let mut names: Vec<&str> = tools.iter().map(|tool| tool["name"].as_str().expect("a name")).collect();

  names.sort_unstable();
  assert_eq!(
    names,
    [
      "text_dashes",
      "text_html_reading",
      "text_language",
      "text_markdown_structure",
      "text_translation_parts",
      "text_web_addresses",
      "text_wording",
    ]
  );

  for tool in tools {
    assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
    assert_eq!(tool["outputSchema"]["type"], "object", "{tool}");
    assert!(
      tool["description"].as_str().is_some_and(|description| !description.is_empty()),
      "{tool}"
    );
  }
}

#[test]
fn each_tool_answers_its_reading_in_the_wire_shape() {
  let folder = Folder::new("call");

  std::fs::write(folder.0.join("WORDING.md"), "## Banned terms\n\n- en · tool strip → toolbar\n")
    .expect("the glossary is written");

  let mut server = Server::started(&folder.0);
  let dash = "Drone survey method \u{2014} high resolution grid";

  assert_eq!(
    answered(&server.call("text_dashes", json!({ "text": dash }))),
    json!({ "text": dash, "found": 1 })
  );
  assert_eq!(
    answered(&server.call("text_dashes", json!({ "text": dash, "replace": true, "wrapped": null }))),
    json!({ "text": "Drone survey method: high resolution grid", "found": 1 })
  );
  assert_eq!(
    answered(&server.call("text_language", json!({ "text": "The run finished." }))),
    json!({ "language": "en" })
  );
  assert_eq!(
    answered(&server.call("text_markdown_structure", json!({ "markdown": "Lead\n# One\nBody" }))),
    json!({
      "headings": [{ "depth": 1, "text": "One", "slug": "one", "line": 2 }],
      "sections": ["Lead\n", "# One\nBody"],
    })
  );
  assert_eq!(
    answered(
      &server.call("text_html_reading", json!({ "html": "<title>Page</title><p>One</p><p>Two</p>" }))
    ),
    json!({ "title": "Page", "lines": ["One", "Two"], "more": false })
  );
  assert_eq!(
    answered(&server.call("text_html_reading", json!({ "html": "<p>One</p><p>Two</p>", "from": 1 }))),
    json!({ "title": null, "lines": ["Two"], "more": false })
  );
  assert_eq!(
    answered(&server.call("text_translation_parts", json!({ "text": "# A\nx\n# B\ny", "limit": 2 }))),
    json!({ "parts": ["# A\nx\n", "# B\ny"] })
  );
  assert_eq!(
    answered(&server.call(
      "text_web_addresses",
      json!({ "text": "See https://example.com/a. And [docs](https://example.com/b)." })
    )),
    json!({ "addresses": ["https://example.com/a", "https://example.com/b"] })
  );
  assert_eq!(
    answered(&server.call("text_wording", json!({ "text": "the `tool strip`\nclick the tool strip" }))),
    json!({
      "swaps": [{ "line": 2, "text": "click the tool strip", "term": "tool strip", "instead": "toolbar", "when": null }],
    })
  );
}

#[test]
fn a_glossary_that_cannot_be_read_is_the_tool_s_error() {
  let folder = Folder::new("missing");
  let mut server = Server::started(&folder.0);
  let result = server.call("text_wording", json!({ "text": "anything" }));

  assert_eq!(result["isError"], true, "{result}");
  assert!(
    result["content"][0]["text"]
      .as_str()
      .is_some_and(|text| text.starts_with("The glossary ")),
    "{result}"
  );
}

#[test]
fn the_server_ends_when_the_harness_closes_its_input() {
  let folder = Folder::new("end");
  let mut server = Server::started(&folder.0);

  drop(server.input.take());

  let status = server.child.wait().expect("the server ends");

  assert!(status.success(), "{status}");
}
