export type Session = 'focus' | 'shortBreak' | 'longBreak';
export type Status = 'idle' | 'running' | 'paused' | 'completed';
export interface Timer {
  session: Session;
  status: Status;
  durationMs: number;
  remainingMs: number;
  completedFocus: number;
  routineCompletedFocus?: number;
  routineComplete?: boolean;
  taskId: string | null;
  wallDeadlineMs: number | null;
}
export interface Task {
  id: string;
  listId: string;
  title: string;
  completed: boolean;
  createdAt: number;
  updatedAt: number;
  completedAt: number | null;
  order: number;
}
export interface TaskList {
  id: string;
  name: string;
}
export interface Settings {
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  sessionsPerCycle: number;
  longBreakEnabled: boolean;
  longBreakInterval: number;
  repeatRoutine: boolean;
  reducedMotion: boolean;
  alwaysOnTop: boolean;
  alarmEnabled: boolean;
  alarm: string;
  alarmVolume: number;
  tickEnabled: boolean;
  tick: string;
  tickVolume: number;
  notifications: boolean;
  startWithWindows: boolean;
  rememberPosition: boolean;
  closeToTray: boolean;
}
export interface Snapshot {
  sampledAt?: number;
  schemaVersion: number;
  revision: number;
  settings: Settings;
  timer: Timer;
  tasks: { lists: TaskList[]; tasks: Task[]; activeListId: string };
  window: { expanded: boolean; position: [number, number] | null };
  history: {
    finishedAt: number;
    session: Session;
    taskId: string | null;
    nextSession?: Session | null;
  }[];
  customAudio: boolean;
  recovery: string | null;
}
export type TaskChange =
  | { action: 'create'; listId: string; title: string }
  | { action: 'update'; id: string; title: string }
  | { action: 'toggle' | 'delete' | 'deleteList' | 'selectList'; id: string }
  | { action: 'reorder'; listId: string; ids: string[] }
  | { action: 'createList'; name: string }
  | { action: 'renameList'; id: string; name: string }
  | { action: 'associate'; id: string | null };
export const defaultSettings: Settings = {
  focusMinutes: 25,
  shortBreakMinutes: 5,
  longBreakMinutes: 15,
  sessionsPerCycle: 4,
  longBreakEnabled: false,
  longBreakInterval: 2,
  repeatRoutine: true,
  reducedMotion: false,
  alwaysOnTop: true,
  alarmEnabled: true,
  alarm: 'chime',
  alarmVolume: 0.65,
  tickEnabled: false,
  tick: 'mechanical',
  tickVolume: 0.2,
  notifications: true,
  startWithWindows: false,
  rememberPosition: true,
  closeToTray: true,
};
