<script lang="ts">
  import { CheckCircle2, Info, TriangleAlert, X } from '@lucide/svelte';
  import { toasts } from '../../state/toasts.svelte';
</script>

{#if toasts.current}
  <div
    class="toast"
    class:error-toast={toasts.current.kind === 'error'}
    role={toasts.current.kind === 'error' ? 'alert' : 'status'}
  >
    {#if toasts.current.kind === 'success'}<CheckCircle2
        size={15}
      />{:else if toasts.current.kind === 'error' || toasts.current.kind === 'warning'}<TriangleAlert
        size={15}
      />{:else}<Info size={15} />{/if}
    <span>{toasts.current.message}</span><button
      aria-label="Dismiss notification"
      onclick={() => toasts.dismiss()}><X size={14} /></button
    >
  </div>
{/if}
