<script lang="ts">
  import {
    Check,
    Crosshair,
    Pencil,
    Trash2,
    ChevronUp,
    ChevronDown,
  } from '@lucide/svelte';
  import IconButton from '../common/IconButton.svelte';
  import type { Task } from '../../types/contracts';
  let {
    task,
    current,
    first,
    last,
    toggle,
    edit,
    remove,
    associate,
    move,
  }: {
    task: Task;
    current: boolean;
    first: boolean;
    last: boolean;
    toggle: () => void;
    edit: (title: string) => Promise<boolean>;
    remove: () => void;
    associate: () => void;
    move: (offset: number) => void;
  } = $props();
  let editing = $state(false);
  let value = $state('');
  async function save() {
    if (await edit(value)) editing = false;
  }
</script>

<li class="task-item" class:done={task.completed} class:selected-task={current}>
  <button
    class="task-check"
    role="checkbox"
    aria-checked={task.completed}
    aria-label={`Complete ${task.title}`}
    onclick={toggle}
    >{#if task.completed}<Check size={13} />{/if}</button
  >
  {#if editing}
    <form
      class="task-edit"
      onsubmit={(e) => {
        e.preventDefault();
        void save();
      }}
    >
      <input
        aria-label="Edit task title"
        bind:value
        maxlength="200"
        onkeydown={(e) => {
          if (e.key === 'Escape') editing = false;
        }}
      /><button type="submit" class="text-button">Save</button>
    </form>
  {:else}
    <span class="task-title" title={task.title}>{task.title}</span>
    <div class="task-actions">
      <IconButton
        label={`Focus on ${task.title}`}
        active={current}
        onclick={associate}><Crosshair size={13} /></IconButton
      >
      <div class="task-extra">
        <IconButton
          label={`Edit ${task.title}`}
          onclick={() => {
            value = task.title;
            editing = true;
          }}><Pencil size={12} /></IconButton
        >
        <IconButton
          label={`Move ${task.title} up`}
          disabled={first}
          onclick={() => move(-1)}><ChevronUp size={12} /></IconButton
        >
        <IconButton
          label={`Move ${task.title} down`}
          disabled={last}
          onclick={() => move(1)}><ChevronDown size={12} /></IconButton
        >
        <IconButton label={`Delete ${task.title}`} onclick={remove}
          ><Trash2 size={12} /></IconButton
        >
      </div>
    </div>
  {/if}
</li>
