import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { api } from '../api/commands';
import type { Snapshot } from '../types/contracts';

class AppState {
  snapshot = $state<Snapshot | null>(null);
  error = $state('');
  busy = $state(false);
  remaining = $state(0);
  private anchor = 0;
  accept(snapshot: Snapshot) {
    if (this.snapshot && snapshot.revision < this.snapshot.revision) return;
    if (
      this.snapshot &&
      snapshot.revision === this.snapshot.revision &&
      (snapshot.sampledAt ?? 0) < (this.snapshot.sampledAt ?? 0)
    )
      return;
    this.snapshot = snapshot;
    this.remaining = snapshot.timer.remainingMs;
    this.anchor = performance.now();
  }
  // This clock only paints a projection; Rust alone completes and advances sessions.
  paint() {
    const timer = this.snapshot?.timer;
    if (timer?.status === 'running')
      this.remaining = Math.max(
        0,
        timer.remainingMs - (performance.now() - this.anchor),
      );
  }
  async run(operation: () => Promise<Snapshot | void>) {
    if (this.busy) return false;
    this.busy = true;
    this.error = '';
    try {
      const result = await operation();
      if (result) this.accept(result);
      return true;
    } catch (error) {
      this.error = String(error);
      return false;
    } finally {
      this.busy = false;
    }
  }
  connect() {
    let closed = false;
    const unlisten: UnlistenFn[] = [];
    const register = async () => {
      try {
        for (const event of [
          'timer:changed',
          'timer:completed',
          'tasks:changed',
          'settings:changed',
          'window:changed',
        ]) {
          const off = await listen<Snapshot>(event, ({ payload }) =>
            this.accept(payload),
          );
          if (closed) off();
          else unlisten.push(off);
        }
        const off = await listen<string>('app:error', ({ payload }) => {
          this.error = payload;
        });
        if (closed) off();
        else unlisten.push(off);
        const snapshot = await api.snapshot();
        if (!closed) this.accept(snapshot);
      } catch (error) {
        if (!closed)
          this.error = `Could not connect to RVNFlow: ${String(error)}`;
      }
    };
    const refresh = () => {
      if (!document.hidden)
        void api
          .snapshot()
          .then((s) => {
            if (!closed) this.accept(s);
          })
          .catch((e) => {
            this.error = String(e);
          });
    };
    void register();
    window.addEventListener('focus', refresh);
    document.addEventListener('visibilitychange', refresh);
    return () => {
      closed = true;
      unlisten.forEach((off) => off());
      window.removeEventListener('focus', refresh);
      document.removeEventListener('visibilitychange', refresh);
    };
  }
}
export const app = new AppState();
