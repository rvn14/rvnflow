<script lang="ts">
  import {
    Clock3,
    Minimize2,
    Settings2,
    X,
    Crosshair,
    Minus,
    PanelBottomClose,
  } from '@lucide/svelte';
  import IconButton from '../common/IconButton.svelte';
  import TimerDisplay from '../timer/TimerDisplay.svelte';
  import TimerControls from '../timer/TimerControls.svelte';
  import TimerProgressDots from '../timer/TimerProgressDots.svelte';
  import TaskList from '../tasks/TaskList.svelte';
  import SettingsPanel from '../settings/SettingsPanel.svelte';
  import { app } from '../../state/app.svelte';
  import { api } from '../../api/commands';
  let settings = $state(false);
  const timer = $derived(app.snapshot!.timer);
  const current = $derived(
    app.snapshot!.tasks.tasks.find((t) => t.id === timer.taskId),
  );
  const phase = $derived(
    timer.session === 'focus'
      ? 'Focus'
      : timer.session === 'longBreak'
        ? 'Long break'
        : 'Short break',
  );
  const completed = $derived(timer.routineCompletedFocus ?? 0);
  const total = $derived(app.snapshot!.settings.sessionsPerCycle);
  const ordinal = $derived(
    Math.min(
      total,
      completed + (timer.session === 'focus' && !timer.routineComplete ? 1 : 0),
    ),
  );
</script>

<div class="expanded-widget">
  <header class="widget-header">
    <span class="brand" data-tauri-drag-region
      ><Clock3 size={15} /><span data-tauri-drag-region
        >RVN<span class="brand-light" data-tauri-drag-region>Flow</span></span
      ></span
    >
    <div class="header-actions">
      <IconButton
        label="Minimize to taskbar"
        onclick={() => app.run(api.minimize)}><Minus size={14} /></IconButton
      >
      <IconButton
        label="Open settings"
        active={settings}
        onclick={() => (settings = !settings)}
        ><Settings2 size={14} /></IconButton
      >
      <IconButton
        label="Collapse widget"
        onclick={() => app.run(() => api.mode(false))}
        ><Minimize2 size={13} /></IconButton
      >
      <IconButton label="Close widget" onclick={() => app.run(api.close)}
        ><X size={15} /></IconButton
      >
    </div>
  </header>
  {#if settings}
    <SettingsPanel close={() => (settings = false)} />
  {:else}
    <section
      class="focus-section"
      class:break-session={timer.session !== 'focus'}
      aria-label="Pomodoro timer"
    >
      <div class="phase-heading">
        <span class="phase-label"
          ><span class="status-dot"></span>{timer.routineComplete
            ? 'Complete'
            : phase}</span
        ><span class="phase-counter"
          >{String(ordinal).padStart(2, '0')}
          <span>/ {String(total).padStart(2, '0')}</span></span
        >
      </div>
      {#if timer.routineComplete}<div class="completion-display">
          All done.<span>{total} focus sessions complete</span>
        </div>{:else}<TimerDisplay remaining={app.remaining} />{/if}
      <TimerProgressDots
        elapsedMs={timer.durationMs - app.remaining}
        durationMs={timer.durationMs}
        running={timer.status === 'running'}
        isBreak={timer.session !== 'focus'}
      />
      <TimerControls />
      {#if current}<div class="current-task" title={current.title}>
          <Crosshair size={11} /><span>{current.title}</span><button
            aria-label="Clear current task"
            onclick={() =>
              app.run(() => api.task({ action: 'associate', id: null }))}
            ><X size={11} /></button
          >
        </div>{/if}
    </section>
    <TaskList />
    <footer>
      <span
        >{timer.routineComplete
          ? 'A well-earned moment to rest.'
          : timer.status === 'running'
            ? 'One thing at a time.'
            : timer.status === 'paused'
              ? 'Paused · your place is saved'
              : `${total} sessions · ${app.snapshot!.settings.focusMinutes} min focus`}</span
      ><IconButton label="Hide to tray" onclick={() => app.run(api.hide)}
        ><PanelBottomClose size={14} /></IconButton
      >
    </footer>
  {/if}
</div>
