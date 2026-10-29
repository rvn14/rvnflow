import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { AppState } from './app.svelte';
import { defaultSettings, type Snapshot } from '../types/contracts';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toasts } from './toasts.svelte';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));
const sample = (): Snapshot => ({
  schemaVersion: 2,
  revision: 1,
  settings: { ...defaultSettings },
  timer: {
    session: 'focus',
    status: 'idle',
    durationMs: 1500000,
    remainingMs: 1500000,
    completedFocus: 0,
    taskId: null,
    wallDeadlineMs: null,
  },
  tasks: {
    lists: [{ id: 'one', name: 'Today' }],
    tasks: [],
    activeListId: 'one',
  },
  window: { expanded: false, position: null },
  history: [],
  customAudio: false,
  recovery: null,
});
const handlers = new Map<string, (event: { payload: unknown }) => void>();
const off = vi.fn();
beforeEach(() => {
  vi.clearAllMocks();
  handlers.clear();
  toasts.dismiss();
  vi.mocked(invoke).mockImplementation(async (command) =>
    command === 'get_startup_status'
      ? { ready: true, loading: false, error: null }
      : command === 'frontend_ready'
        ? undefined
        : sample(),
  );
  vi.mocked(listen).mockImplementation(async (event, handler) => {
    handlers.set(
      event as string,
      handler as (event: { payload: unknown }) => void,
    );
    return off;
  });
});
afterEach(() => {
  toasts.dismiss();
  vi.restoreAllMocks();
});
describe('native event lifecycle', () => {
  it('loads a snapshot and unregisters all listeners on disconnect', async () => {
    const state = new AppState();
    const disconnect = state.connect();
    await vi.waitFor(() => expect(state.snapshot).not.toBeNull());
    expect(handlers.size).toBe(8);
    disconnect();
    expect(off).toHaveBeenCalledTimes(8);
  });
  it('cleans partial initialization and reconnects after a listener error', async () => {
    vi.mocked(listen).mockRejectedValueOnce(
      new Error('event service unavailable'),
    );
    const state = new AppState();
    const disconnect = state.connect();
    await vi.waitFor(() =>
      expect(state.error).toContain('event service unavailable'),
    );
    state.retryConnection();
    await vi.waitFor(() => expect(state.snapshot).not.toBeNull());
    disconnect();
    expect(off).toHaveBeenCalledTimes(8);
  });
  it('shows a completion once and ignores stale completion payloads', async () => {
    vi.spyOn(document, 'hasFocus').mockReturnValue(true);
    const state = new AppState();
    const disconnect = state.connect();
    await vi.waitFor(() => expect(state.snapshot).not.toBeNull());
    const completed = sample();
    completed.revision = 2;
    completed.timer.session = 'shortBreak';
    completed.timer.durationMs = completed.timer.remainingMs = 300000;
    completed.history = [
      {
        finishedAt: 1000,
        session: 'focus',
        taskId: null,
        nextSession: 'shortBreak',
      },
    ];
    handlers.get('timer:completed')!({ payload: completed });
    expect(state.remaining).toBe(300000);
    expect(toasts.current?.message).toContain('Short break started');
    toasts.dismiss();
    handlers.get('timer:completed')!({ payload: completed });
    expect(toasts.current).toBeNull();
    const later = structuredClone(completed);
    later.revision = 3;
    later.timer.session = 'focus';
    handlers.get('timer:changed')!({ payload: later });
    handlers.get('timer:completed')!({ payload: completed });
    expect(state.snapshot?.timer.session).toBe('focus');
    disconnect();
  });
});
