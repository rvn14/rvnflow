<script lang="ts">
  import type { Snippet } from 'svelte';
  let {
    label,
    onclick,
    children,
    disabled = false,
    active = false,
  }: {
    label: string;
    onclick?: () => void;
    children: Snippet;
    disabled?: boolean;
    active?: boolean;
  } = $props();
</script>

<button
  class="icon-button"
  class:active
  title={label}
  aria-label={label}
  {onclick}
  {disabled}>{@render children()}</button
>
