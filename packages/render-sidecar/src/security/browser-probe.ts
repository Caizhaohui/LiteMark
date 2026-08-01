/**
 * Browser discovery for PDF export (DEVELOPMENT_PLAN.md §9.2).
 *
 * Detection order:
 *   1. An explicit caller-supplied browser path.
 *   2. Microsoft Edge stable (Chromium-based; present on Windows 10/11).
 *   3. Google Chrome stable.
 *   4. Return null -> caller maps to BROWSER_NOT_FOUND.
 *
 * We use `chrome-paths` (a transitive dependency of crossnote) for the
 * Chrome-family fallback and hard-code the well-known Edge install locations
 * on Windows. No silent downloads, no bundled Chromium.
 */

import { existsSync } from "node:fs";
import * as path from "node:path";
import { ExternalToolStatus } from "@litemark/shared-protocol";

/** Known Windows install locations for Microsoft Edge stable. */
const EDGE_CANDIDATES_WIN32 = [
  path.join(
    process.env.PROGRAMFILES_X86 ?? "C:\\Program Files (x86)",
    "Microsoft",
    "Edge",
    "Application",
    "msedge.exe",
  ),
  path.join(
    process.env.PROGRAMFILES ?? "C:\\Program Files",
    "Microsoft",
    "Edge",
    "Application",
    "msedge.exe",
  ),
  path.join(
    process.env.LOCALAPPDATA ?? "",
    "Microsoft",
    "Edge",
    "Application",
    "msedge.exe",
  ),
];

/**
 * Resolve a browser executable path for Puppeteer. Returns null if none found.
 * `preferred` (user setting) wins if it exists.
 */
export async function findBrowserPath(
  preferred?: string | null,
): Promise<string | null> {
  if (preferred && preferred.trim() !== "" && existsSync(preferred)) {
    return preferred;
  }

  if (process.platform === "win32") {
    for (const candidate of EDGE_CANDIDATES_WIN32) {
      if (candidate && existsSync(candidate)) {
        return candidate;
      }
    }
  }

  // Chrome-family fallback via chrome-paths (crossnote dependency).
  try {
    const mod = await import("chrome-paths");
    const chrome = mod.chrome ?? mod.chromium ?? mod.chromeCanary;
    if (chrome && existsSync(chrome)) {
      return chrome;
    }
  } catch {
    // chrome-paths not installed or threw — fall through to "not found".
  }

  return null;
}

/**
 * Produce an `ExternalToolStatus` entry describing browser availability, for
 * the `probeExternalTools` and `getCapabilities` results.
 */
export async function probeBrowser(
  preferred?: string | null,
): Promise<ExternalToolStatus> {
  const resolved = await findBrowserPath(preferred);
  return {
    name: resolved && /msedge/i.test(resolved) ? "Microsoft Edge" : "Google Chrome",
    available: resolved !== null,
    path: resolved,
    version: null,
  };
}
