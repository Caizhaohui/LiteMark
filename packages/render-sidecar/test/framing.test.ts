import { describe, it, expect } from "vitest";
import {
  LineBuffer,
  encodeFrame,
  decodeLine,
} from "../src/protocol/framing.js";
import { ok, err } from "@litemark/shared-protocol";

describe("LineBuffer", () => {
  it("yields complete lines and buffers the remainder", () => {
    const buf = new LineBuffer();
    expect(buf.push("hello\nworld")).toEqual(["hello"]);
    expect(buf.push("\n")).toEqual(["world"]);
    expect(buf.flush()).toEqual([]);
  });

  it("handles CRLF by leaving a trailing \\r in the line (decodeLine trims it)", () => {
    const buf = new LineBuffer();
    const lines = buf.push('{"id":"1"}\r\n');
    expect(lines).toEqual(['{"id":"1"}\r']);
    // decodeLine trims whitespace (including \r) before parsing JSON.
    expect(decodeLine(lines[0])).toEqual({ id: "1" });
  });

  it("accumulates chunks across calls until a newline arrives", () => {
    const buf = new LineBuffer();
    expect(buf.push("part1-")).toEqual([]);
    expect(buf.push("part2")).toEqual([]);
    expect(buf.push("\n")).toEqual(["part1-part2"]);
  });

  it("flush returns unterminated trailing content once", () => {
    const buf = new LineBuffer();
    buf.push("leftover");
    expect(buf.flush()).toEqual(["leftover"]);
    expect(buf.flush()).toEqual([]);
  });
});

describe("encodeFrame", () => {
  it("serializes a success response on one line", () => {
    const frame = encodeFrame(ok("1", { version: "0.1", crossnoteVersion: "0.9.31" }));
    expect(frame).not.toContain("\n");
    expect(JSON.parse(frame)).toMatchObject({ id: "1", ok: true });
  });

  it("serializes an error response on one line", () => {
    const frame = encodeFrame(
      err("2", { code: "RENDER_FAILED", message: "boom", details: null }),
    );
    expect(frame).not.toContain("\n");
    expect(JSON.parse(frame)).toMatchObject({ id: "2", ok: false });
  });
});

describe("decodeLine", () => {
  it("returns null for blank/whitespace lines", () => {
    expect(decodeLine("")).toBeNull();
    expect(decodeLine("   \t  ")).toBeNull();
  });

  it("parses valid JSON", () => {
    expect(decodeLine('{"id":"1","method":"ping","params":{}}')).toEqual({
      id: "1",
      method: "ping",
      params: {},
    });
  });

  it("throws on invalid JSON", () => {
    expect(() => decodeLine("{not json}")).toThrow(SyntaxError);
  });
});
