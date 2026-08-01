/**
 * JSON Lines framing for the LiteMark sidecar IPC.
 *
 * Protocol rule (DEVELOPMENT_PLAN.md §5.3): every message is exactly one JSON
 * object on a single line. stdout is reserved for protocol messages; all
 * logging MUST go to stderr. The functions here enforce the framing invariant
 * and never touch stdio directly, so they are trivially unit-testable.
 */

import { type SidecarResponse, type SidecarEvent } from "@litemark/shared-protocol";

/**
 * Serialize a response/event to a single JSON Lines frame (one line, no
 * embedded newlines). Throws if the payload serializes to something containing
 * a raw newline, which would corrupt the framing.
 */
export function encodeFrame(message: SidecarResponse | SidecarEvent): string {
  const json = JSON.stringify(message);
  if (json.includes("\n")) {
    // Should be impossible for JSON.stringify output, but guard the invariant.
    throw new Error(
      "PROTOCOL_INVALID: serialized message contains a newline, refusing to emit a malformed frame",
    );
  }
  return json;
}

/**
 * Decode a single line into a parsed value. Returns `null` for blank lines so
 * callers can tolerate trailing newlines. Throws on invalid JSON.
 */
export function decodeLine(line: string): unknown {
  const trimmed = line.trim();
  if (trimmed === "") {
    return null;
  }
  return JSON.parse(trimmed);
}

/**
 * A buffered line reader that accumulates arbitrary chunked stdin writes and
 * yields complete, newline-terminated lines. Carries any trailing partial line
 * across calls. Pure with respect to its own buffer state (no I/O).
 */
export class LineBuffer {
  private buffer = "";

  /** Feed raw text; return the complete lines found within it. */
  push(chunk: string): string[] {
    this.buffer += chunk;
    const lines: string[] = [];
    let newlineIndex = this.buffer.indexOf("\n");
    while (newlineIndex !== -1) {
      const line = this.buffer.slice(0, newlineIndex);
      this.buffer = this.buffer.slice(newlineIndex + 1);
      lines.push(line);
      newlineIndex = this.buffer.indexOf("\n");
    }
    return lines;
  }

  /** Flush any unterminated trailing content (e.g. on stream end). */
  flush(): string[] {
    if (this.buffer === "") {
      return [];
    }
    const remaining = this.buffer;
    this.buffer = "";
    return [remaining];
  }
}
