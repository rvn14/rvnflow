<script lang="ts">
  import { Play, Pause, RotateCcw, SkipForward, Square } from '@lucide/svelte';
  import IconButton from '../common/IconButton.svelte';
  import { api } from '../../api/commands';
  import { app } from '../../state/app.svelte';
  let { compact = false }: { compact?: boolean } = $props();
  const status = $derived(app.snapshot?.timer.status);
  const startLabel = $derived(
    app.snapshot?.timer.session === 'focus' ? 'Start focus' : 'Start break',
  );
  const label = $derived(
    status === 'running'
      ? 'Pause session'
      : status === 'paused'
        ? 'Resume session'
        : status === 'completed'
          ? 'Start new routine'
          : startLabel,
  );
</script>

<div class="timer-controls" class:compact-controls={compact}>
  {#if !compact}<IconButton
      label={status === 'running' || status === 'paused'
        ? 'Stop routine'
        : 'Reset routine'}
      disabled={app.busy}
      onclick={() => app.run(() => api.timer('reset'))}
      >{#if status === 'running' || status === 'paused'}<Square
          size={14}
        />{:else}<RotateCcw size={16} />{/if}</IconButton
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
              ? 'New routine'
              : startLabel}</span
      >{/if}
  </button>
  {#if !compact}<IconButton
      label="Skip session"
      disabled={app.busy}
      onclick={() => app.run(() => api.timer('skip'))}
      ><SkipForward size={17} /></IconButton
    >{/if}
</div>
