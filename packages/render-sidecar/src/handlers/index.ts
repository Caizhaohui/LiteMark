/**
 * Sidecar method handlers.
 *
 * Each handler narrows its request to the concrete params type from
 * shared-protocol, delegates to the crossnote adapter, and returns the typed
 * result. Security-sensitive flags are clamped in the adapter, not here.
 */

import type { SidecarMethod } from "@litemark/shared-protocol";
import type { Handler } from "../dispatch.js";
import { CROSSNOTE_VERSION } from "../crossnote-adapter/notebook-pool.js";
import { renderMarkdownText } from "../crossnote-adapter/render.js";
import { exportHtml } from "../crossnote-adapter/export-html.js";
import { exportPdf } from "../crossnote-adapter/export-pdf.js";
import { probeBrowser } from "../security/browser-probe.js";

/** App + sidecar build version. Matches package.json via the build. */
export const SIDECAR_VERSION = "0.1.0";

// In-memory session registry. For M0 a session is just an opaque id we track so
// closeSession can report success; crossnote Notebooks are cached per
// notebookPath, not per session.
const sessions = new Set<string>();

export const ping: Handler = async (request) => {
  if (request.method !== "ping") throw new Error("internal: wrong handler");
  const params = request.params as { sentAt?: string };
  return {
    version: SIDECAR_VERSION,
    crossnoteVersion: CROSSNOTE_VERSION,
    receivedAt: params.sentAt,
  };
};

export const getCapabilities: Handler = async (request) => {
  if (request.method !== "getCapabilities") throw new Error("internal: wrong handler");
  // Probe browser so the host can warn about PDF export availability early.
  const browser = await probeBrowser(undefined);
  return {
    methods: [
      "ping",
      "getCapabilities",
      "createSession",
      "closeSession",
      "render",
      "exportHtml",
      "exportPdf",
      "cancelJob",
      "probeExternalTools",
      "shutdown",
    ],
    crossnoteVersion: CROSSNOTE_VERSION,
    safeDefaults: {
      enableScriptExecution: false as const,
      enableHTML5Embed: false as const,
      protocolsWhiteList: "http://, https://, mailto:, tel:",
    },
    externalTools: [browser],
  };
};

export const createSession: Handler = async (request) => {
  if (request.method !== "createSession") throw new Error("internal: wrong handler");
  const params = request.params as { sessionId: string };
  if (!params.sessionId) {
    throw { code: "PROTOCOL_INVALID", message: "sessionId is required", details: null };
  }
  sessions.add(params.sessionId);
  return { sessionId: params.sessionId, ok: true as const };
};

export const closeSession: Handler = async (request) => {
  if (request.method !== "closeSession") throw new Error("internal: wrong handler");
  const params = request.params as { sessionId: string };
  sessions.delete(params.sessionId);
  return { sessionId: params.sessionId, ok: true as const };
};

export const render: Handler = async (request) => {
  if (request.method !== "render") throw new Error("internal: wrong handler");
  const params = request.params as {
    sessionId: string;
    markdown: string;
    logicalFilePath?: string | null;
    revision?: number;
    options?: object;
  };
  const started = Date.now();
  const { html, toc } = await renderMarkdownText({
    markdown: params.markdown,
    logicalFilePath: params.logicalFilePath ?? null,
    options: params.options as never,
  });
  return {
    html,
    toc,
    diagnostics: [],
    renderMs: Date.now() - started,
  };
};

export const exportHtmlHandler: Handler = async (request) => {
  if (request.method !== "exportHtml") throw new Error("internal: wrong handler");
  const params = request.params as {
    sessionId: string;
    markdown: string;
    logicalFilePath?: string | null;
    offline?: boolean;
    outputPath: string;
    jobId?: string;
    options?: object;
  };
  if (!params.outputPath) {
    throw { code: "PATH_NOT_AUTHORIZED", message: "outputPath is required", details: null };
  }
  return exportHtml({
    markdown: params.markdown,
    logicalFilePath: params.logicalFilePath ?? null,
    offline: params.offline,
    outputPath: params.outputPath,
    jobId: params.jobId,
    options: params.options as never,
  });
};

export const exportPdfHandler: Handler = async (request) => {
  if (request.method !== "exportPdf") throw new Error("internal: wrong handler");
  const params = request.params as {
    sessionId: string;
    markdown: string;
    logicalFilePath?: string | null;
    outputPath: string;
    page?: object;
    browserPath?: string | null;
    jobId?: string;
    options?: object;
  };
  if (!params.outputPath) {
    throw { code: "PATH_NOT_AUTHORIZED", message: "outputPath is required", details: null };
  }
  return exportPdf({
    markdown: params.markdown,
    logicalFilePath: params.logicalFilePath ?? null,
    outputPath: params.outputPath,
    page: params.page as never,
    browserPath: params.browserPath ?? null,
    jobId: params.jobId,
  });
};

export const cancelJob: Handler = async (request) => {
  if (request.method !== "cancelJob") throw new Error("internal: wrong handler");
  const params = request.params as { jobId: string };
  if (!params.jobId) {
    throw { code: "PROTOCOL_INVALID", message: "jobId is required", details: null };
  }
  const { cancelJob: cancel } = await import("../jobs.js");
  const known = await cancel(params.jobId);
  return { jobId: params.jobId, cancelled: true as const, known };
};

export const probeExternalTools: Handler = async (request) => {
  if (request.method !== "probeExternalTools") throw new Error("internal: wrong handler");
  const browser = await probeBrowser(undefined);
  return { tools: [browser] };
};

export const shutdown: Handler = async () => {
  // The entry point listens for the shutdown result and exits cleanly.
  return { version: SIDECAR_VERSION, crossnoteVersion: CROSSNOTE_VERSION };
};

export const handlers: Readonly<Record<SidecarMethod, Handler>> = {
  ping,
  getCapabilities,
  createSession,
  closeSession,
  render,
  exportHtml: exportHtmlHandler,
  exportPdf: exportPdfHandler,
  cancelJob,
  probeExternalTools,
  shutdown,
} as const;
