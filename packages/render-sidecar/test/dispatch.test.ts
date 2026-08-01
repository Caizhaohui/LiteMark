import { describe, it, expect } from "vitest";
import { dispatch, sidecarError } from "../src/dispatch.js";
import type { SidecarRequest, SidecarMethod } from "@litemark/shared-protocol";
import type { Handler } from "../src/dispatch.js";

// Build a minimal request object for tests.
function req(method: SidecarMethod, id = "1", params: object = {}): SidecarRequest {
  return { id, method, params } as SidecarRequest;
}

describe("dispatch", () => {
  it("returns an ok envelope with the handler result", async () => {
    const handlers = {
      ping: (async () => ({ version: "x" })) as Handler,
    } as unknown as Record<SidecarMethod, Handler>;
    const res = await dispatch(req("ping"), handlers);
    expect(res).toEqual({ id: "1", ok: true, result: { version: "x" } });
  });

  it("wraps a thrown SidecarError into an error envelope", async () => {
    const handlers = {
      render: (async () => {
        throw sidecarError("RENDER_FAILED", "nope", { extra: 1 });
      }) as Handler,
    } as unknown as Record<SidecarMethod, Handler>;
    const res = await dispatch(req("render"), handlers);
    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.error.code).toBe("RENDER_FAILED");
      expect(res.error.message).toBe("nope");
      expect(res.error.details).toEqual({ extra: 1 });
    }
  });

  it("converts an arbitrary thrown value into RENDER_FAILED", async () => {
    const handlers = {
      render: (async () => {
        throw new Error("unexpected");
      }) as Handler,
    } as unknown as Record<SidecarMethod, Handler>;
    const res = await dispatch(req("render"), handlers);
    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.error.code).toBe("RENDER_FAILED");
      expect(res.error.message).toContain("unexpected");
    }
  });

  it("keeps the request id through the response", async () => {
    const handlers = {
      ping: (async () => ({ version: "x" })) as Handler,
    } as unknown as Record<SidecarMethod, Handler>;
    const res = await dispatch(req("ping", "abc-123"), handlers);
    expect(res.id).toBe("abc-123");
  });
});
