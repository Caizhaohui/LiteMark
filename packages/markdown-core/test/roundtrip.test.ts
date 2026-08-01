import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import {
  hybridRoundtrip,
  normalizeMarkdown,
  findRawRegions,
} from "../src/index.js";

const fixturesDir = join(dirname(fileURLToPath(import.meta.url)), "fixtures");

describe("normalizeMarkdown", () => {
  it("collapses blank lines and trims trailing spaces", () => {
    expect(normalizeMarkdown("a  \n\n\n\nb\n")).toBe("a\n\nb\n");
  });
});

describe("hybridRoundtrip golden fixtures", () => {
  const files = readdirSync(fixturesDir)
    .filter((f) => f.endsWith(".md"))
    .sort();

  it("has at least 100 fixtures", () => {
    expect(files.length).toBeGreaterThanOrEqual(100);
  });

  for (const file of files) {
    it(`roundtrips ${file}`, () => {
      const md = readFileSync(join(fixturesDir, file), "utf8");
      const result = hybridRoundtrip(md);
      if (!result.ok) {
        // Document known losses rather than silently rewriting user files.
        expect(result.risks.length).toBeGreaterThan(0);
      } else {
        expect(normalizeMarkdown(result.serialized)).toBe(normalizeMarkdown(md));
      }
    });
  }
});

describe("raw regions", () => {
  it("detects unknown fence languages as raw", () => {
    const md = "```plantuml\n@startuml\n@enduml\n```\n";
    const regions = findRawRegions(md);
    expect(regions.some((r) => r.kind === "fence")).toBe(true);
  });

  it("detects front matter", () => {
    const md = "---\ntitle: x\n---\n\n# Hi\n";
    const regions = findRawRegions(md);
    expect(regions.some((r) => r.kind === "front_matter")).toBe(true);
  });
});
