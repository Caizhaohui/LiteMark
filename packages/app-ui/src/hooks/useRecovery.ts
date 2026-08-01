/**
 * Crash-recovery hook. On mount, queries pending recovery snapshots and, if
 * any exist, surfaces them to the UI via the returned state. The user can
 * restore or discard each (or all).
 */

import { useCallback, useEffect, useState } from "react";
import type { RecoveryEntry } from "@litemark/shared-protocol";
import * as cmd from "../services/tauriCommands";

export interface UseRecovery {
  pending: RecoveryEntry[];
  dismiss: () => void;
  restoreOne: (recoveryKey: string) => Promise<string | null>;
  discardOne: (recoveryKey: string) => Promise<void>;
  discardAll: () => Promise<void>;
}

export function useRecovery(): UseRecovery {
  const [pending, setPending] = useState<RecoveryEntry[]>([]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const entries = await cmd.getPendingRecovery();
        if (!cancelled) setPending(entries);
      } catch {
        // Non-fatal: if recovery lookup fails, just start fresh.
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const dismiss = useCallback(() => setPending([]), []);

  const restoreOne = useCallback(async (recoveryKey: string): Promise<string | null> => {
    try {
      const id = await cmd.restoreRecoverySnapshot(recoveryKey);
      setPending((prev) => prev.filter((e) => e.recoveryKey !== recoveryKey));
      return id;
    } catch {
      return null;
    }
  }, []);

  const discardOne = useCallback(async (recoveryKey: string) => {
    await cmd.discardRecoverySnapshot(recoveryKey);
    setPending((prev) => prev.filter((e) => e.recoveryKey !== recoveryKey));
  }, []);

  const discardAll = useCallback(async () => {
    await cmd.discardAllRecovery();
    setPending([]);
  }, []);

  return { pending, dismiss, restoreOne, discardOne, discardAll };
}
