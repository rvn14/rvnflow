<script lang="ts">
  import { Maximize2, CircleAlert } from '@lucide/svelte';
  import TimerDisplay from '../timer/TimerDisplay.svelte';
  import TimerControls from '../timer/TimerControls.svelte';
  import TimerProgressDots from '../timer/TimerProgressDots.svelte';
  import IconButton from '../common/IconButton.svelte';
  import { app } from '../../state/app.svelte';
  import { api } from '../../api/commands';
  const timer = $derived(app.snapshot!.timer);
</script>

<div class="mini-widget">
  <div class="mini-heading">
    <span
      data-tauri-drag-region
      class="mini-session"
      title={app.completionMessage || undefined}
      ><span class="status-dot" class:break-status={timer.session !== 'focus'}
      ></span>{app.completionMessage
        ? 'SESSION DONE'
        : timer.session === 'focus'
          ? 'FOCUS'
          : 'BREATHE'}</span
    ><IconButton
      label="Expand widget"
      onclick={() => app.run(() => api.mode(true))}
      >{#if app.error}<CircleAlert size={13} />{:else}<Maximize2
          size={12}
        />{/if}</IconButton
    >
  </div>
  <div class="mini-clock">
    <TimerDisplay remaining={app.remaining} /><TimerControls compact />
  </div>
  <TimerProgressDots
    elapsedMs={timer.durationMs - app.remaining}
    durationMs={timer.durationMs}
    running={timer.status === 'running'}
    isBreak={timer.session !== 'focus'}
  />
</div>
