<script lang="ts">
  import { Check, ChevronDown } from '@lucide/svelte';
  let {
    label,
    value,
    options,
    onchange,
    disabled = false,
  }: {
    label: string;
    value: string;
    options: { value: string; label: string }[];
    onchange: (value: string) => void;
    disabled?: boolean;
  } = $props();
  const id = $props.id();
  let trigger: HTMLButtonElement;
  let popup: HTMLDivElement;
  let open = $state(false);
  let active = $state(0);
  let search = '';
  let searchAt = 0;
  const selected = $derived(options.find((option) => option.value === value));

  function position() {
    const rect = trigger.getBoundingClientRect();
    const below = window.innerHeight - rect.bottom - 17;
    const above = rect.top - 17;
    const down = below >= 120 || below >= above;
    const height = Math.max(
      36,
      Math.min(options.length * 34 + 12, 240, down ? below : above),
    );
    const width = Math.min(Math.max(rect.width, 170), window.innerWidth - 24);
    popup.style.left = `${Math.max(12, Math.min(rect.left, window.innerWidth - width - 12))}px`;
    popup.style.top = `${down ? rect.bottom + 5 : Math.max(8, rect.top - height - 5)}px`;
    popup.style.width = `${width}px`;
    popup.style.maxHeight = `${height}px`;
  }
  function reveal() {
    if (disabled || !options.length) return;
    active = Math.max(
      0,
      options.findIndex((option) => option.value === value),
    );
    position();
    popup.showPopover();
    open = true;
    scrollToActive();
  }
  function close() {
    popup.hidePopover();
    open = false;
  }
  function choose(index: number) {
    const option = options[index];
    if (!option) return;
    close();
    trigger.focus({ preventScroll: true });
    // The parent commits the selection. Failed task-list saves retain the old value.
    if (option.value !== value) onchange(option.value);
  }
  function scrollToActive() {
    queueMicrotask(() =>
      document
        .getElementById(`${id}-${active}`)
        ?.scrollIntoView({ block: 'nearest' }),
    );
  }
  function keydown(event: KeyboardEvent) {
    if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
      event.preventDefault();
      if (!open) {
        reveal();
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') return;
      }
      active =
        event.key === 'Home'
          ? 0
          : event.key === 'End'
            ? options.length - 1
            : (active + (event.key === 'ArrowDown' ? 1 : -1) + options.length) %
              options.length;
      scrollToActive();
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      if (open) choose(active);
      else reveal();
    } else if (event.key === 'Escape' && open) {
      event.preventDefault();
      event.stopPropagation();
      close();
    } else if (event.key === 'Tab' && open) {
      close();
    } else if (
      event.key.length === 1 &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey
    ) {
      event.preventDefault();
      if (!open) reveal();
      const now = performance.now();
      search =
        now - searchAt < 600
          ? search + event.key.toLocaleLowerCase()
          : event.key.toLocaleLowerCase();
      searchAt = now;
      const index = options.findIndex((option) =>
        option.label.toLocaleLowerCase().startsWith(search),
      );
      if (index >= 0) {
        active = index;
        scrollToActive();
      }
    }
  }
  $effect(() => {
    if (!open) return;
    // A top-layer popover escapes the widget's rounded clipping and scroll panels.
    // Close on movement so the menu never floats away from its trigger.
    const dismiss = () => close();
    const onScroll = (event: Event) => {
      if (event.target !== popup) close();
    };
    window.addEventListener('resize', dismiss);
    document.addEventListener('scroll', onScroll, true);
    window.addEventListener('blur', dismiss);
    return () => {
      window.removeEventListener('resize', dismiss);
      document.removeEventListener('scroll', onScroll, true);
      window.removeEventListener('blur', dismiss);
    };
  });
</script>

<button
  class="select-trigger"
  class:select-open={open}
  type="button"
  role="combobox"
  aria-label={label}
  aria-expanded={open}
  aria-haspopup="listbox"
  aria-controls={`${id}-options`}
  aria-activedescendant={open ? `${id}-${active}` : undefined}
  disabled={disabled || !options.length}
  bind:this={trigger}
  onkeydown={keydown}
  onclick={() => (open ? close() : reveal())}
>
  <span>{selected?.label ?? 'Choose…'}</span><ChevronDown
    size={13}
    aria-hidden="true"
  />
</button>
<div
  class="select-popover"
  id={`${id}-options`}
  role="listbox"
  aria-label={label}
  popover="auto"
  bind:this={popup}
  ontoggle={() => (open = popup.matches(':popover-open'))}
>
  {#each options as option, index (option.value)}
    <button
      class="select-option"
      class:option-active={index === active}
      type="button"
      role="option"
      aria-selected={option.value === value}
      id={`${id}-${index}`}
      tabindex="-1"
      onpointerdown={(event) => event.preventDefault()}
      onpointermove={() => (active = index)}
      onclick={() => choose(index)}
    >
      <span>{option.label}</span>{#if option.value === value}<Check
          size={13}
          aria-hidden="true"
        />{/if}
    </button>
  {/each}
</div>
