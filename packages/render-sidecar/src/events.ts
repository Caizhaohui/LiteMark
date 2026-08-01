/**
 * stdout event emitter for the sidecar protocol.
 * Events are JSON Lines with `{ event, payload }` (no `id` / `ok`).
 */

import { encodeFrame } from "./protocol/framing.js";
import type { ExportProgressPayload, ExportProgressStage } from "@litemark/shared-protocol";

function writeFrame(obj: unknown): void {
  process.stdout.write(encodeFrame(obj as never) + "\n", "utf8");
}

export function emitEvent(event: string, payload: unknown): void {
  writeFrame({ event, payload });
}

export function emitExportProgress(
  jobId: string,
  stage: ExportProgressStage,
  progress: number,
  message?: string,
): void {
  if (!jobId) return;
  const payload: ExportProgressPayload = {
    jobId,
    stage,
    progress: Math.min(1, Math.max(0, progress)),
    message,
  };
  emitEvent("exportProgress", payload);
}
