export {
  normalizeLineEndings,
  normalizeMarkdown,
  normalizeRawBlock,
} from "./normalize.js";

export {
  findRawRegions,
  protectRawRegions,
  restoreRawRegions,
  type RawRegion,
} from "./raw-blocks.js";

export {
  hybridRoundtrip,
  serializeFromHybrid,
  type RoundtripResult,
} from "./roundtrip.js";
