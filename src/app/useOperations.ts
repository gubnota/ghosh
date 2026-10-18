import { useCallback, useEffect, useRef, useState } from "react";
import type { ApprovalDecisionRequest, OperationsSnapshot } from "../generated/contracts";
import { api, desktop, errorMessage } from "./ipc";

export function useOperations() {
  const [snapshot, setSnapshot] = useState<OperationsSnapshot | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [mutating, setMutating] = useState(false);
  const mutation = useRef(false);
  const refresh = useRef<() => Promise<void>>(async () => {});
  useEffect(() => {
    if (!desktop) return;
    let alive = true, fetching = false, queued = false;
    let stop: (() => void) | undefined;
    const fetch = async () => {
      if (!alive) return;
      if (fetching) { queued = true; return; }
      fetching = true;
      try {
        do {
          queued = false;
          const next = await api.operations();
          if (alive) {
            setSnapshot(previous => ({
              jobs: previous && previous.jobs.sequence > next.jobs.sequence ? previous.jobs : next.jobs,
              approvals: previous && previous.approvals.sequence > next.approvals.sequence ? previous.approvals : next.approvals,
            }));
            setLoadError(null);
          }
        } while (alive && queued);
      } catch (e) { if (alive) setLoadError(errorMessage(e)); }
      finally { fetching = false; }
    };
    refresh.current = fetch;
    // Subscribe before the initial snapshot; a full snapshot repairs missed hints.
    void (async () => {
      try {
        const dispose = await api.onOperations(() => { void fetch(); });
        if (!alive) { dispose(); return; }
        stop = dispose;
      } catch (e) { if (alive) setLoadError(errorMessage(e)); }
      await fetch();
    })();
    const visible = () => { if (document.visibilityState === "visible") void fetch(); };
    const timer = window.setInterval(visible, 2000);
    document.addEventListener("visibilitychange", visible);
    return () => {
      alive = false; stop?.(); window.clearInterval(timer);
      document.removeEventListener("visibilitychange", visible);
      refresh.current = async () => {};
    };
  }, []);
  const perform = useCallback(async (action: () => Promise<void>) => {
    if (mutation.current || !desktop) return false;
    mutation.current = true; setMutating(true); setError(null);
    try { await action(); await refresh.current(); return true; }
    catch (e) { await refresh.current(); setError(errorMessage(e)); return false; }
    finally { mutation.current = false; setMutating(false); }
  }, []);
  return { snapshot, error: error || loadError, mutating, refresh: () => refresh.current(),
    cancel: (id: string) => perform(() => api.cancelJob(id)),
    decide: (request: ApprovalDecisionRequest) => perform(() => api.decide(request)),
  };
}
export type Operations = ReturnType<typeof useOperations>;
