import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { api } from '../api/commands';
import type { Snapshot } from '../types/contracts';
import { toasts } from './toasts.svelte';

export class AppState {
  snapshot = $state<Snapshot | null>(null);
  error = $state('');
  busy = $state(false);
  remaining = $state(0);
  startupError = $state('');
  completionMessage = $state('');
  private completionTimer: ReturnType<typeof setTimeout> | undefined;
  private lastCompletion = '';
  private anchor = 0;
  private reconnect: (() => void) | undefined;
  retryConnection() {
    this.startupError = '';
    this.error = '';
    this.reconnect?.();
  }
  accept(snapshot: Snapshot) {
    if (this.snapshot && snapshot.revision < this.snapshot.revision)
      return false;
    if (
      this.snapshot &&
      snapshot.revision === this.snapshot.revision &&
      (snapshot.sampledAt ?? 0) < (this.snapshot.sampledAt ?? 0)
    )
      return false;
    this.snapshot = snapshot;
    this.remaining = snapshot.timer.remainingMs;
    this.anchor = performance.now();
    return true;
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
  async run(operation: () => Promise<Snapshot | void>, success?: string) {
    if (this.busy) return false;
    this.busy = true;
    this.error = '';
    try {
      const result = await operation();
      if (result) this.accept(result);
      if (success) toasts.show(success, 'success');
      return true;
    } catch (error) {
      this.error = String(error);
      toasts.show(this.error, 'error');
      return false;
    } finally {
      this.busy = false;
    }
  }
  connect() {
    let closed = false;
    let announced = false;
    const announce = () => {
      if (!announced && !closed) {
        announced = true;
        void api.frontendReady().catch(() => {});
      }
    };
    const unlisten: UnlistenFn[] = [];
    let registering = false;
    const register = async () => {
      if (registering || closed) return;
      registering = true;
      unlisten.splice(0).forEach((off) => off());
      try {
        for (const event of [
          'timer:changed',
          'timer:completed',
          'tasks:changed',
          'settings:changed',
          'window:changed',
          'app:ready',
        ]) {
          const off = await listen<Snapshot>(event, ({ payload }) => {
            if (closed) return;
            if (!this.accept(payload)) return;
            announce();
            if (event === 'timer:completed') this.completed(payload);
          });
          if (closed) off();
          else unlisten.push(off);
        }
        const off = await listen<string>('app:error', ({ payload }) => {
          this.error = payload;
          if (!document.hidden) toasts.show(payload, 'error');
        });
        if (closed) off();
        else unlisten.push(off);
        const startupOff = await listen<{ error: string | null }>(
          'startup:changed',
          ({ payload }) => {
            if (!closed) {
              this.startupError = payload.error ?? '';
              if (payload.error) this.snapshot = null;
            }
          },
        );
        if (closed) startupOff();
        else unlisten.push(startupOff);
        const status = await api.startup();
        if (status.ready) {
          const snapshot = await api.snapshot();
          if (!closed) {
            this.accept(snapshot);
            announce();
          }
        } else if (!closed) this.startupError = status.error ?? '';
      } catch (error) {
        if (!closed)
          this.error = `Could not connect to RVNFlow: ${String(error)}`;
        unlisten.splice(0).forEach((off) => off());
      } finally {
        registering = false;
      }
    };
    this.reconnect = () => {
      void register();
    };
    const refresh = () => {
      if (!document.hidden && this.snapshot)
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
      clearTimeout(this.completionTimer);
      this.reconnect = undefined;
      unlisten.forEach((off) => off());
      window.removeEventListener('focus', refresh);
      document.removeEventListener('visibilitychange', refresh);
    };
  }
  private completed(snapshot: Snapshot) {
    const record = snapshot.history.at(-1);
    if (!record) return;
    const key = `${record.finishedAt}:${record.session}`;
    if (key === this.lastCompletion) return;
    this.lastCompletion = key;
    const minutes = Math.round(snapshot.timer.durationMs / 60000);
    const message = snapshot.timer.routineComplete
      ? `Routine complete · ${snapshot.settings.sessionsPerCycle} focus sessions finished.`
      : record.session === 'focus'
        ? `Focus complete · ${snapshot.timer.session === 'longBreak' ? 'Long' : 'Short'} break started — ${minutes} minutes.`
        : 'Break finished · Your next focus session has started.';
    this.completionMessage = message;
    clearTimeout(this.completionTimer);
    this.completionTimer = setTimeout(
      () => (this.completionMessage = ''),
      6500,
    );
    if (!document.hidden && document.hasFocus()) toasts.show(message, 'info');
  }
}
export const app = new AppState();
