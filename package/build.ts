// Makes the package's WebAssembly build into `wasm/` (docs/design.md, 「One implementation」): the crate compiled
// with its `wasm` feature alone under its `wasm` profile, its binding's glue and types written by `wasm-bindgen`, the
// module made smaller by Binaryen's `wasm-opt`, its bytes written into a JavaScript module the package
// instantiates on import, and the hash of the crate's sources the build was made from.

import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";

const root = join(import.meta.dir, "..");
const out = join(import.meta.dir, "wasm");

// The file the hash of the crate's sources is kept in, beside the build made from them.
export const sourceHashFile = join(out, "sources.sha256");

// Every file under a folder, as paths from the repository's root.
function filesUnder(folder: string): string[] {
  return readdirSync(join(root, folder), { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => relative(root, join(entry.parentPath, entry.name)));
}

// What the build reads: the crate's manifest, its lockfile and its sources, in one order.
function sources(): string[] {
  return ["Cargo.toml", "Cargo.lock", ...filesUnder("src")].sort();
}

// The hash of what the build reads, each file by its path and its bytes.
export function sourceHash(): string {
  const hash = createHash("sha256");

  for (const path of sources()) {
    hash.update(`${path}\0`);
    hash.update(readFileSync(join(root, path)));
    hash.update("\0");
  }

  return hash.digest("hex");
}

function run(command: string[]): string {
  const done = Bun.spawnSync(command, { cwd: root, stdout: "pipe", stderr: "inherit" });

  if (done.exitCode !== 0) {
    throw new Error(`${command.join(" ")} failed with exit code ${done.exitCode}.`);
  }

  return done.stdout.toString();
}

function build(): void {
  run([
    "cargo", "rustc", "--lib", "--profile", "wasm", "--target", "wasm32-unknown-unknown", "--no-default-features",
    "--features", "wasm", "--crate-type", "cdylib",
  ]);

  const target: string = JSON.parse(run(["cargo", "metadata", "--format-version", "1", "--no-deps"])).target_directory;
  const made = mkdtempSync(join(tmpdir(), "bmo-text-wasm-"));

  try {
    run([
      "wasm-bindgen", join(target, "wasm32-unknown-unknown", "wasm", "bmo_text.wasm"), "--out-dir", made, "--target",
      "web", "--omit-default-module-path",
    ]);
    run([
      "bun", join(import.meta.dir, "node_modules", "binaryen", "bin", "wasm-opt"), "-Oz", "--strip-debug",
      "--strip-producers", join(made, "bmo_text_bg.wasm"), "-o", join(made, "bmo_text.wasm"),
    ]);

    rmSync(out, { recursive: true, force: true });
    mkdirSync(out);
    copyFileSync(join(made, "bmo_text.js"), join(out, "bmo_text.js"));
    copyFileSync(join(made, "bmo_text.d.ts"), join(out, "bmo_text.d.ts"));

    const bytes = readFileSync(join(made, "bmo_text.wasm")).toString("base64");

    writeFileSync(join(out, "bytes.js"), `// The package's WebAssembly module, written by build.ts.\nexport const bytes = "${bytes}";\n`);
    writeFileSync(join(out, "bytes.d.ts"), "// The package's WebAssembly module, in base64.\nexport declare const bytes: string;\n");
    writeFileSync(sourceHashFile, `${sourceHash()}\n`);
  } finally {
    rmSync(made, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  build();
}
