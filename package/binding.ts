// The crate built to WebAssembly (docs/design.md, 「One implementation」), compiled and instantiated the first time
// any module of the package is imported, so nothing is awaited before a call. The module's bytes are a JavaScript
// module's text, which Bun, Node and every bundler read as they are. The instantiated build is the value every
// module calls through, so a bundler that drops a module's statements when nothing it exports is used never
// leaves a call ahead of the instantiation.

import * as glue from "./wasm/bmo_text.js";
import { bytes } from "./wasm/bytes.js";

function decoded(base64: string): Uint8Array {
  const binary = atob(base64);
  const decoded = new Uint8Array(binary.length);

  for (let at = 0; at < binary.length; at += 1) {
    decoded[at] = binary.charCodeAt(at);
  }

  return decoded;
}

function instantiated(): typeof glue {
  glue.initSync({ module: decoded(bytes) });

  return glue;
}

export const binding = instantiated();
