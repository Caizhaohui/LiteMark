/**
 * LiteMark render sidecar — process entry point.
 *
 * Lifecycle (DEVELOPMENT_PLAN.md §5.3):
 *  - Reads JSON Lines requests from stdin (one object per line).
 *  - Writes JSON Lines responses/events to stdout.
 *  - All logging goes to stderr; stdout is reserved for protocol messages.
 *  - Emits a `ready` event once handlers are wired.
 *  - Exits cleanly on `shutdown` method, SIGTERM, or stdin close.
 */

import * as readline from "node:readline";
import {
  isKnownMethod,
  isSidecarRequest,
  err as errEnvelope,
  type RequestId,
  type SidecarRequest,
} from "@litemark/shared-protocol";
import { LineBuffer, encodeFrame } from "./protocol/framing.js";
import { dispatch } from "./dispatch.js";
import { handlers } from "./handlers/index.js";
import { SIDECAR_VERSION } from "./handlers/index.js";
import { emitEvent } from "./events.js";

const stdoutWrite = (frame: string): void => {
  // writeSync guarantees ordering and avoids backpressure races on stdout.
  // Append the newline that delimits frames.
  process.stdout.write(frame + "\n", "utf8");
};

const log = {
  info: (msg: string) => process.stderr.write(`[sidecar:info] ${msg}\n`),
  warn: (msg: string) => process.stderr.write(`[sidecar:warn] ${msg}\n`),
  error: (msg: string) => process.stderr.write(`[sidecar:error] ${msg}\n`),
};

let shuttingDown = false;

/** Tracks in-flight requests so we don't exit mid-work when stdin closes. */
let inflight = 0;
let stdinClosed = false;
let resolveDrained: (() => void) | null = null;
const drainedPromise = (): Promise<void> =>
  new Promise((resolve) => {
    resolveDrained = resolve;
  });

/** Mark a request as in-flight; returns a function to call when it finishes. */
function trackInflight(): () => void {
  inflight++;
  return () => {
    inflight--;
    if (inflight === 0 && stdinClosed && resolveDrained) {
      resolveDrained();
    }
  };
}

function gracefulShutdown(reason: string): void {
  if (shuttingDown) return;
  shuttingDown = true;
  log.info(`shutting down: ${reason}`);
  // Flush is handled by Node on exit. Give in-flight requests a moment.
  setImmediate(() => {
    process.exit(0);
  });
}

async function handleLine(raw: string): Promise<void> {
  const done = trackInflight();
  try {
    await handleLineInner(raw);
  } finally {
    done();
  }
}

async function handleLineInner(raw: string): Promise<void> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    stdoutWrite(
      encodeFrame(
        errEnvelope("0" as RequestId, {
          code: "PROTOCOL_INVALID",
          message: "Request line is not valid JSON",
          details: { line: raw.slice(0, 200) },
        }),
      ),
    );
    return;
  }

  // Structural validation against the whitelist.
  if (!parsed || typeof parsed !== "object") {
    stdoutWrite(
      encodeFrame(
        errEnvelope("0" as RequestId, {
          code: "PROTOCOL_INVALID",
          message: "Request must be a JSON object",
          details: null,
        }),
      ),
    );
    return;
  }

  const candidate = parsed as { id?: unknown; method?: unknown };
  const id = typeof candidate.id === "string" ? (candidate.id as RequestId) : null;
  if (!id) {
    stdoutWrite(
      encodeFrame(
        errEnvelope("0" as RequestId, {
          code: "PROTOCOL_INVALID",
          message: "Request must include a string `id`",
          details: null,
        }),
      ),
    );
    return;
  }

  if (!isKnownMethod(candidate.method)) {
    stdoutWrite(
      encodeFrame(
        errEnvelope(id, {
          code: "PROTOCOL_INVALID",
          message: `Unknown or forbidden method: ${String(candidate.method)}`,
          details: null,
        }),
      ),
    );
    return;
  }

  // `isSidecarRequest` also confirms `params` is an object.
  if (!isSidecarRequest(parsed)) {
    stdoutWrite(
      encodeFrame(
        errEnvelope(id, {
          code: "PROTOCOL_INVALID",
          message: "Request shape invalid (expected id, method, params)",
          details: null,
        }),
      ),
    );
    return;
  }

  const request = parsed as SidecarRequest;
  const response = await dispatch(request, handlers);

  // The `shutdown` method triggers a graceful exit AFTER we reply.
  if (request.method === "shutdown") {
    stdoutWrite(encodeFrame(response));
    gracefulShutdown("shutdown method");
    return;
  }

  stdoutWrite(encodeFrame(response));
}

async function main(): Promise<void> {
  log.info(`LiteMark render sidecar v${SIDECAR_VERSION} starting`);

  // Emit the ready event so the Rust host knows we accept requests.
  emitEvent("ready", { version: SIDECAR_VERSION });

  const rl = readline.createInterface({ input: process.stdin, terminal: false });

  // readline already yields complete, newline-delimited lines. We feed them
  // through LineBuffer as well so the framing layer is the single code path
  // exercised in tests and in production.
  const buffer = new LineBuffer();

  rl.on("line", (line) => {
    // Do not let one bad line take down the process.
    for (const completed of buffer.push(line + "\n")) {
      handleLine(completed).catch((e) => {
        log.error(
          `unexpected error handling line: ${e instanceof Error ? e.message : String(e)}`,
        );
      });
    }
  });

  await new Promise<void>((resolve) => {
    rl.on("close", () => resolve());
  });

  // stdin closed (host went away). Flush any trailing partial line, then wait
  // for all in-flight requests to finish before shutting down — otherwise a
  // slow export would be killed mid-flight when the host stops sending.
  stdinClosed = true;
  for (const line of buffer.flush()) {
    if (line.trim()) {
      try {
        await handleLine(line);
      } catch {
        /* ignore final flush errors */
      }
    }
  }

  if (inflight > 0) {
    log.info(`waiting for ${inflight} in-flight request(s) to finish`);
    await drainedPromise();
  }

  gracefulShutdown("stdin closed");
}

process.on("SIGTERM", () => gracefulShutdown("SIGTERM"));
process.on("SIGINT", () => gracefulShutdown("SIGINT"));
process.on("uncaughtException", (e) => {
  log.error(`uncaughtException: ${e.message}`);
});
process.on("unhandledRejection", (reason) => {
  log.error(`unhandledRejection: ${String(reason)}`);
});

main().catch((e) => {
  log.error(`fatal: ${e instanceof Error ? e.message : String(e)}`);
  process.exit(1);
});
