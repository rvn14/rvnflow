<script lang="ts">
  import { onMount } from 'svelte';
  import { X, CircleAlert } from '@lucide/svelte';
  import MiniWidget from './lib/components/widget/MiniWidget.svelte';
  import ExpandedWidget from './lib/components/widget/ExpandedWidget.svelte';
  import { app } from './lib/state/app.svelte';
  import { api } from './lib/api/commands';
  onMount(() => app.connect());
  $effect(() => {
    if (app.snapshot?.timer.status !== 'running') return;
    const clock = window.setInterval(() => {
      if (!document.hidden) app.paint();
    }, 1000);
    return () => window.clearInterval(clock);
  });
</script>

<main
  class="widget-shell"
  class:expanded={app.snapshot?.window.expanded}
  class:reduced-motion={app.snapshot?.settings.reducedMotion}
>
  {#if app.snapshot}
    {#if app.snapshot.window.expanded}<ExpandedWidget />{:else}<MiniWidget
      />{/if}
    {#if (app.error || app.snapshot.recovery) && app.snapshot.window.expanded}<div
        class="error-banner"
        role="alert"
      >
        <CircleAlert size={15} /><span
          >{app.error || app.snapshot.recovery}</span
        ><button
          aria-label="Dismiss message"
          onclick={() => {
            app.error = '';
            if (app.snapshot) app.snapshot.recovery = null;
          }}><X size={14} /></button
        >
      </div>{/if}
  {:else}<div class="loading-state">
      <strong>RVNFlow</strong>
      <p>{app.error || 'Making space for focus…'}</p>
      {#if app.error}<button
          class="text-button"
          onclick={() => app.run(api.snapshot)}>Try again</button
        >{/if}
    </div>{/if}
</main>
