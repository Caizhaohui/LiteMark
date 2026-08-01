/**
 * Ambient type declarations for `chrome-paths`, an optional transitive
 * dependency (via crossnote) used only as a Chrome-family browser fallback in
 * PDF export. The package ships no its own types.
 */
declare module "chrome-paths" {
  export const chrome: string | undefined;
  export const chromeCanary: string | undefined;
  export const chromium: string | undefined;
}
