<script lang="ts">
  import { Play, Pause, RotateCcw, SkipForward } from '@lucide/svelte';
  import IconButton from '../common/IconButton.svelte';
  import { api } from '../../api/commands';
  import { app } from '../../state/app.svelte';
  let { compact = false }: { compact?: boolean } = $props();
  const status = $derived(app.snapshot?.timer.status);
  const label = $derived(
    status === 'running'
      ? 'Pause session'
      : status === 'paused'
        ? 'Resume session'
        : status === 'completed'
          ? 'Start next session'
          : 'Start focus',
  );
</script>

<div class="timer-controls" class:compact-controls={compact}>
  {#if !compact}<IconButton
      label="Reset session"
      disabled={app.busy}
      onclick={() => app.run(() => api.timer('reset'))}
      ><RotateCcw size={16} /></IconButton
    >{/if}
  <button
    class="play-button"
    aria-label={label}
    title={label}
    disabled={app.busy}
    onclick={() =>
      app.run(() => api.timer(status === 'running' ? 'pause' : 'start'))}
  >
    {#if status === 'running'}<Pause
        size={compact ? 17 : 19}
        fill="currentColor"
      />{:else}<Play size={compact ? 17 : 19} fill="currentColor" />{/if}
    {#if !compact}<span
        >{status === 'running'
          ? 'Pause'
          : status === 'paused'
            ? 'Resume'
            : status === 'completed'
              ? 'Next session'
              : 'Start focus'}</span
      >{/if}
  </button>
  {#if !compact}<IconButton
      label="Skip session"
      disabled={app.busy}
      onclick={() => app.run(() => api.timer('skip'))}
      ><SkipForward size={17} /></IconButton
    >{/if}
</div>
