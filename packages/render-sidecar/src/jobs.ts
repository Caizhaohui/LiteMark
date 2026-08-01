/**
 * In-flight export job registry for cancel support (M3).
 *
 * PDF/HTML exports register a cleanup (kill browser, rm temp dirs). `cancelJob`
 * marks the job cancelled and runs cleanup so no browser process is left behind
 * (DEVELOPMENT_PLAN.md M3 acceptance).
 */

export type JobCleanup = () => void | Promise<void>;

interface JobRecord {
  cancelled: boolean;
  cleanups: JobCleanup[];
}

const jobs = new Map<string, JobRecord>();

function ensure(jobId: string): JobRecord {
  let rec = jobs.get(jobId);
  if (!rec) {
    rec = { cancelled: false, cleanups: [] };
    jobs.set(jobId, rec);
  }
  return rec;
}

/** Register (or re-register) a job id before starting work. */
export function beginJob(jobId: string): void {
  jobs.set(jobId, { cancelled: false, cleanups: [] });
}

/** Attach a cleanup callback (e.g. kill browser child). Runs on cancel or end. */
export function onJobCleanup(jobId: string, cleanup: JobCleanup): void {
  ensure(jobId).cleanups.push(cleanup);
}

/** True if cancelJob has been requested for this id. */
export function isJobCancelled(jobId: string | undefined | null): boolean {
  if (!jobId) return false;
  return jobs.get(jobId)?.cancelled === true;
}

/**
 * Throw a structured EXPORT_CANCELLED error if the job was cancelled.
 * Call between export stages.
 */
export function throwIfCancelled(jobId: string | undefined | null): void {
  if (isJobCancelled(jobId)) {
    throw {
      code: "EXPORT_CANCELLED",
      message: "Export was cancelled",
      details: { jobId },
    };
  }
}

/**
 * Mark a job cancelled and run all cleanups. Returns whether the job was known.
 */
export async function cancelJob(jobId: string): Promise<boolean> {
  const rec = jobs.get(jobId);
  if (!rec) {
    // Still record the cancel so a race (cancel before beginJob) is honored.
    jobs.set(jobId, { cancelled: true, cleanups: [] });
    return false;
  }
  rec.cancelled = true;
  const cleanups = rec.cleanups.splice(0);
  for (const fn of cleanups) {
    try {
      await fn();
    } catch {
      /* best-effort */
    }
  }
  return true;
}

/** Run cleanups and drop the job record after success or failure. */
export async function endJob(jobId: string | undefined | null): Promise<void> {
  if (!jobId) return;
  const rec = jobs.get(jobId);
  if (!rec) return;
  const cleanups = rec.cleanups.splice(0);
  for (const fn of cleanups) {
    try {
      await fn();
    } catch {
      /* best-effort */
    }
  }
  jobs.delete(jobId);
}
