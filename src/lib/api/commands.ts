import { invoke } from '@tauri-apps/api/core';
import type {
  Session,
  Settings,
  Snapshot,
  TaskChange,
} from '../types/contracts';
export const api = {
  snapshot: () => invoke<Snapshot>('get_app_snapshot'),
  timer: (
    action: 'start' | 'pause' | 'resume' | 'reset' | 'skip' | 'select',
    session?: Session,
  ) => invoke<Snapshot>('timer_action', { action, session }),
  task: (change: TaskChange) => invoke<Snapshot>('task_action', { change }),
  settings: (settings: Settings) =>
    invoke<Snapshot>('settings_update', { settings }),
  mode: (expanded: boolean) =>
    invoke<Snapshot>('window_set_mode', { expanded }),
  close: () => invoke<void>('window_close'),
  minimize: () => invoke<void>('window_minimize'),
  hide: () => invoke<void>('window_hide'),
  quit: () => invoke<void>('app_quit'),
  startup: () =>
    invoke<{ ready: boolean; loading: boolean; error: string | null }>(
      'get_startup_status',
    ),
  retryStartup: () => invoke<void>('startup_retry'),
  frontendReady: () => invoke<void>('frontend_ready'),
  preview: (kind: 'alarm' | 'tick', settings?: Settings) =>
    invoke<void>('audio_preview', { kind, settings }),
  importAudio: () => invoke<Snapshot>('audio_import_custom'),
};
