/**
 * Method dispatcher — maps a validated sidecar request to its handler.
 *
 * Only methods present in the static `SIDECAR_METHODS` whitelist are
 * dispatchable; the framing layer rejects everything else with PROTOCOL_INVALID
 * (DEVELOPMENT_PLAN.md §5.3). There is intentionally no `exec`/`shell`/
 * `runCommand` entry.
 */

import {
  isSidecarRequest,
  err as errEnvelope,
  ok as okEnvelope,
  type RequestId,
  type SidecarRequest,
  type SidecarMethod,
  type SidecarError,
  type SidecarResponse,
  type SidecarSuccessResponse,
} from "@litemark/shared-protocol";

/**
 * A handler takes the typed request and returns either its result or throws a
 * `SidecarError`. The dispatcher wraps thrown SidecarErrors into error
 * envelopes and unexpected throws into RENDER_FAILED.
 */
export type HandlerResult = unknown;
export type Handler = (request: SidecarRequest) => Promise<HandlerResult>;

/** Build a SidecarError with a stable code + message. */
export function sidecarError(
  code: SidecarError["code"],
  message: string,
  details: unknown = null,
): SidecarError {
  return { code, message, details };
}

/** Is the thrown value a SidecarError we built ourselves? */
function isSidecarError(value: unknown): value is SidecarError {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as SidecarError).code === "string" &&
    typeof (value as SidecarError).message === "string" &&
    "details" in value
  );
}

/**
 * Dispatch a single validated request, returning a fully-formed response
 * envelope (success or error). Never throws.
 */
export async function dispatch(
  request: SidecarRequest,
  handlers: Readonly<Record<SidecarMethod, Handler>>,
): Promise<SidecarResponse> {
  const id: RequestId = request.id;
  try {
    const handler = handlers[request.method];
    // `isSidecarRequest` already guarantees the method is known, but double-check
    // defensively in case a caller bypassed validation.
    if (!handler) {
      return errEnvelope(id, sidecarError("PROTOCOL_INVALID", `Unknown method: ${request.method}`));
    }
    const result = await handler(request);
    // Handlers return a narrow result type per method; the ok() helper is
    // generic over the method->result map. Cast through the success envelope
    // to bridge the dynamically-dispatched handler result to the right type.
    return okEnvelope(id, result as never) as SidecarSuccessResponse<SidecarMethod>;
  } catch (e) {
    if (isSidecarError(e)) {
      return errEnvelope(id, e);
    }
    // Handlers may throw Error with name set to a known code (e.g. BROWSER_NOT_FOUND).
    if (e instanceof Error && e.name && e.name !== "Error") {
      return errEnvelope(id, sidecarError(e.name, e.message));
    }
    const message = e instanceof Error ? e.message : String(e);
    // Prefer EXPORT_FAILED for export-looking messages; otherwise RENDER_FAILED.
    const code = /export|browser|pdf|html/i.test(message) ? "EXPORT_FAILED" : "RENDER_FAILED";
    return errEnvelope(id, sidecarError(code, message));
  }
}

/** Re-export the validator so the entry point imports framing + dispatch together. */
export { isSidecarRequest };
