<script lang="ts">
  import { formatTime } from '../../utils/time';
  let { remaining }: { remaining: number } = $props();
</script>

<div
  class="timer-display"
  role="timer"
  aria-label="Time remaining"
  aria-live="off"
>
  {formatTime(remaining)}
</div>
