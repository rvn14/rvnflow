import { beforeEach, afterEach, describe, it, expect, vi } from 'vitest';
import { cleanup, render, screen, fireEvent } from '@testing-library/svelte';
import { app } from '../../state/app.svelte';
import { defaultSettings, type Snapshot } from '../../types/contracts';
import MiniWidget from './MiniWidget.svelte';
import ExpandedWidget from './ExpandedWidget.svelte';
import SettingsPanel from '../settings/SettingsPanel.svelte';
import TaskItem from '../tasks/TaskItem.svelte';
import TimerProgressDots from '../timer/TimerProgressDots.svelte';
import { invoke } from '@tauri-apps/api/core';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const snapshot = (): Snapshot => ({
  schemaVersion: 1,
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
    lists: [{ id: 'today', name: 'Today' }],
    tasks: [],
    activeListId: 'today',
  },
  window: { expanded: false, position: null },
  history: [],
  customAudio: false,
  recovery: null,
});
beforeEach(() => {
  app.snapshot = null;
  app.accept(snapshot());
  app.busy = false;
  app.error = '';
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(snapshot());
});
afterEach(cleanup);
describe('widget interactions', () => {
  it('renders compact timer and starts through native IPC', async () => {
    render(MiniWidget);
    expect(screen.getByRole('timer')).toHaveTextContent('25:00');
    await fireEvent.click(screen.getByRole('button', { name: 'Start focus' }));
    expect(invoke).toHaveBeenCalledWith('timer_action', {
      action: 'start',
      session: undefined,
    });
  });
  it('renders expanded empty tasks and separate task progress', () => {
    render(ExpandedWidget);
    expect(screen.getByText('Make room for one thing.')).toBeInTheDocument();
    expect(
      screen.getByRole('progressbar', { name: 'Task completion' }),
    ).toHaveAttribute('aria-valuenow', '0');
    expect(
      screen.getByRole('button', { name: 'Collapse widget' }),
    ).toBeInTheDocument();
  });
  it('pauses a running session', async () => {
    const state = snapshot();
    state.timer.status = 'running';
    app.accept(state);
    render(MiniWidget);
    await fireEvent.click(
      screen.getByRole('button', { name: 'Pause session' }),
    );
    expect(invoke).toHaveBeenCalledWith('timer_action', {
      action: 'pause',
      session: undefined,
    });
  });
  it('renders all completed dots and stops the pulse when paused', () => {
    const { container } = render(TimerProgressDots, {
      elapsedMs: 100,
      durationMs: 100,
      running: false,
    });
    expect(container.querySelectorAll('.dot.complete')).toHaveLength(10);
    expect(container.querySelector('.pulse')).toBeNull();
  });
  it('saves a changed setting through IPC', async () => {
    render(SettingsPanel, { close: vi.fn() });
    await fireEvent.click(
      screen.getByRole('checkbox', { name: 'Reduced motion' }),
    );
    await fireEvent.click(screen.getByRole('button', { name: 'Save changes' }));
    expect(invoke).toHaveBeenCalledWith('settings_update', {
      settings: { ...defaultSettings, reducedMotion: true },
    });
  });
  it('emits task completion and focus actions', async () => {
    const toggle = vi.fn(),
      associate = vi.fn();
    render(TaskItem, {
      task: {
        id: 'one',
        listId: 'today',
        title: 'Read chapter',
        completed: false,
        createdAt: 1,
        updatedAt: 1,
        completedAt: null,
        order: 0,
      },
      current: false,
      first: true,
      last: true,
      toggle,
      associate,
      edit: vi.fn(),
      remove: vi.fn(),
      move: vi.fn(),
    });
    await fireEvent.click(screen.getByRole('checkbox'));
    expect(toggle).toHaveBeenCalledOnce();
    await fireEvent.click(
      screen.getByRole('button', { name: 'Focus on Read chapter' }),
    );
    expect(associate).toHaveBeenCalledOnce();
  });
  it('rejects stale state events', () => {
    const newer = snapshot();
    newer.revision = 10;
    newer.timer.remainingMs = 1000;
    app.accept(newer);
    app.accept(snapshot());
    expect(app.remaining).toBe(1000);
  });
  it('rejects an older countdown sample at the same revision', () => {
    const recent = snapshot();
    recent.sampledAt = 100;
    recent.timer.remainingMs = 500;
    app.accept(recent);
    const older = snapshot();
    older.sampledAt = 90;
    app.accept(older);
    expect(app.remaining).toBe(500);
  });
  it('reports IPC failures without optimistic data loss', async () => {
    vi.mocked(invoke).mockRejectedValue('Disk full');
    render(MiniWidget);
    await fireEvent.click(screen.getByRole('button', { name: 'Start focus' }));
    expect(app.error).toBe('Disk full');
    expect(app.snapshot?.timer.status).toBe('idle');
  });
});
