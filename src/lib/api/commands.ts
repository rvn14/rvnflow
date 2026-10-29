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
  preview: (kind: 'alarm' | 'tick') => invoke<void>('audio_preview', { kind }),
  importAudio: () => invoke<Snapshot>('audio_import_custom'),
};
