import type { DeepPartial, LocaleId, MessageTree } from "../types";
import { zhCN } from "./zh-CN";
import { zhTW } from "./zh-TW";
import { ja } from "./ja";

/** Non-English catalogs. Missing keys fall back to English via deepMerge. */
export const catalogs: Record<Exclude<LocaleId, "en">, DeepPartial<MessageTree>> = {
  "zh-CN": zhCN,
  "zh-TW": zhTW,
  ja,
};
