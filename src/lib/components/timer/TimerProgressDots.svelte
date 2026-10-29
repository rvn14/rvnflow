<script lang="ts">
  import { dotProgress } from '../../utils/progress';
  let {
    elapsedMs,
    durationMs,
    running = false,
    isBreak = false,
  }: {
    elapsedMs: number;
    durationMs: number;
    running?: boolean;
    isBreak?: boolean;
  } = $props();
  const dots = $derived(dotProgress(elapsedMs, durationMs));
</script>

<div
  class="timer-dots"
  class:break-dots={isBreak}
  role="progressbar"
  aria-label="Session progress"
  aria-valuenow={Math.round(dots.progress * 100)}
  aria-valuemin="0"
  aria-valuemax="100"
>
  {#each Array.from({ length: 10 }) as _, index (index)}
    <span
      class="dot"
      class:complete={index < dots.completed}
      class:current={index === dots.completed && dots.progress < 1}
      class:pulse={index === dots.completed && running}
      aria-hidden="true"
    ></span>
  {/each}
</div>
