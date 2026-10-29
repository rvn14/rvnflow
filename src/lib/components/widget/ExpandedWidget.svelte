<script lang="ts">
  import {
    Minimize2,
    Settings2,
    X,
    Moon,
    Sparkles,
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
  import type { Session } from '../../types/contracts';
  let settings = $state(false);
  const timer = $derived(app.snapshot!.timer);
  const current = $derived(
    app.snapshot!.tasks.tasks.find((t) => t.id === timer.taskId),
  );
  const sessions: { id: Session; label: string }[] = [
    { id: 'focus', label: 'Focus' },
    { id: 'shortBreak', label: 'Short break' },
    { id: 'longBreak', label: 'Long break' },
  ];
</script>

<div class="expanded-widget">
  <header class="widget-header">
    <span class="brand" data-tauri-drag-region
      ><span class="brand-symbol" data-tauri-drag-region>◷</span> RVN<span
        class="brand-light"
        data-tauri-drag-region>Flow</span
      ></span
    >
    <div class="header-actions">
      <IconButton
        label="Minimize to taskbar"
        onclick={() => app.run(api.minimize)}><Minus size={15} /></IconButton
      >
      <IconButton label="Settings" onclick={() => (settings = !settings)}
        ><Settings2 size={14} /></IconButton
      >
      <IconButton
        label="Collapse widget"
        onclick={() => app.run(() => api.mode(false))}
        ><Minimize2 size={14} /></IconButton
      ><IconButton label="Close widget" onclick={() => app.run(api.close)}
        ><X size={15} /></IconButton
      >
    </div>
  </header>
  {#if settings}<SettingsPanel close={() => (settings = false)} />{:else}
    <section class="focus-section" aria-label="Pomodoro timer">
      <div class="session-tabs" role="group" aria-label="Session type">
        {#each sessions as session}<button
            class:chosen={timer.session === session.id}
            aria-pressed={timer.session === session.id}
            onclick={() => app.run(() => api.timer('select', session.id))}
            >{session.label}</button
          >{/each}
      </div>
      <div class="session-caption">
        {#if timer.session === 'focus'}<Sparkles size={12} /> A LITTLE SPACE TO FOCUS{:else}<Moon
            size={12}
          /> REST IS PART OF THE RHYTHM{/if}
      </div>
      <TimerDisplay remaining={app.remaining} />
      <TimerProgressDots
        elapsedMs={timer.durationMs - app.remaining}
        durationMs={timer.durationMs}
        running={timer.status === 'running'}
        isBreak={timer.session !== 'focus'}
      />
      <div class="cycle-status" aria-live="polite">
        {app.completionMessage
          ? app.completionMessage
          : timer.status === 'paused'
            ? 'Take your time. Your place is saved.'
            : `${timer.session === 'focus' ? 'Focus' : 'Break'} · ${timer.completedFocus % app.snapshot!.settings.sessionsPerCycle || (timer.session !== 'focus' && timer.completedFocus ? app.snapshot!.settings.sessionsPerCycle : 0)} / ${app.snapshot!.settings.sessionsPerCycle} focused this cycle`}
      </div>
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
        ><span class="status-dot"></span>{timer.status === 'running'
          ? 'One thing at a time.'
          : 'Find your flow.'}</span
      >
      <div class="footer-actions">
        <IconButton label="Hide to tray" onclick={() => app.run(api.hide)}
          ><PanelBottomClose size={15} /></IconButton
        ><IconButton label="Open settings" onclick={() => (settings = true)}
          ><Settings2 size={16} /></IconButton
        >
      </div>
    </footer>
  {/if}
</div>
