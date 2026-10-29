<script lang="ts">
  import { Plus, ChevronDown, Pencil, Trash2 } from '@lucide/svelte';
  import IconButton from '../common/IconButton.svelte';
  import TaskItem from './TaskItem.svelte';
  import { api } from '../../api/commands';
  import { app } from '../../state/app.svelte';
  import { taskProgress } from '../../utils/progress';
  let title = $state('');
  let listName = $state('');
  let listEdit = $state<'create' | 'rename' | null>(null);
  let confirmDelete = $state(false);
  let open = $state(true);
  const data = $derived(app.snapshot!.tasks);
  const list = $derived(data.lists.find((l) => l.id === data.activeListId)!);
  const tasks = $derived(
    data.tasks
      .filter((t) => t.listId === data.activeListId)
      .sort((a, b) => a.order - b.order),
  );
  const progress = $derived(taskProgress(tasks));
  async function add() {
    if (
      await app.run(
        () => api.task({ action: 'create', listId: list.id, title }),
        'Task added successfully.',
      )
    )
      title = '';
  }
  async function saveList() {
    if (
      await app.run(() =>
        api.task(
          listEdit === 'create'
            ? { action: 'createList', name: listName }
            : { action: 'renameList', id: list.id, name: listName },
        ),
      )
    )
      listEdit = null;
  }
  function move(index: number, offset: number) {
    const ids = tasks.map((t) => t.id);
    [ids[index], ids[index + offset]] = [ids[index + offset], ids[index]];
    void app.run(() => api.task({ action: 'reorder', listId: list.id, ids }));
  }
</script>

<section
  class="tasks-section"
  class:tasks-collapsed={!open}
  aria-label="Task lists"
>
  <div class="section-heading">
    <button
      class="task-section-toggle"
      aria-expanded={open}
      onclick={() => (open = !open)}
      ><span>Tasks</span><ChevronDown
        size={12}
        class={open ? '' : 'rotated'}
      /></button
    >
    <span class="count"
      >{progress.completed}<span> / {progress.total}</span></span
    >
  </div>
  {#if open}
    <div class="list-toolbar">
      <select
        aria-label="Active task list"
        value={list.id}
        onchange={(e) => {
          confirmDelete = false;
          listEdit = null;
          void app.run(() =>
            api.task({ action: 'selectList', id: e.currentTarget.value }),
          );
        }}
        >{#each data.lists as item}<option value={item.id}>{item.name}</option
          >{/each}</select
      >
      <IconButton
        label="Create task list"
        onclick={() => {
          listName = '';
          listEdit = 'create';
        }}><Plus size={14} /></IconButton
      >
      <IconButton
        label="Rename task list"
        onclick={() => {
          listName = list.name;
          listEdit = 'rename';
        }}><Pencil size={13} /></IconButton
      >
      <IconButton
        label="Delete task list"
        disabled={data.lists.length < 2}
        onclick={() => (confirmDelete = true)}><Trash2 size={13} /></IconButton
      >
    </div>
    {#if listEdit}
      <form
        class="inline-form"
        onsubmit={(e) => {
          e.preventDefault();
          void saveList();
        }}
      >
        <input
          aria-label="List name"
          bind:value={listName}
          placeholder="Name your list"
          maxlength="200"
          required
        /><button class="text-button" type="submit">Save</button><button
          class="text-button"
          type="button"
          onclick={() => (listEdit = null)}>Cancel</button
        >
      </form>
    {/if}
    {#if confirmDelete}
      <div class="confirmation" role="alert">
        <p>Delete “{list.name}” and all its tasks?</p>
        <button
          class="danger text-button"
          onclick={async () => {
            if (
              await app.run(() =>
                api.task({ action: 'deleteList', id: list.id }),
              )
            )
              confirmDelete = false;
          }}>Delete list</button
        ><button class="text-button" onclick={() => (confirmDelete = false)}
          >Keep list</button
        >
      </div>
    {/if}
    <div
      class="task-progress"
      role="progressbar"
      aria-label="Task completion"
      aria-valuenow={progress.percent}
      aria-valuemin="0"
      aria-valuemax="100"
    >
      <span style:width={`${progress.percent}%`}></span>
    </div>
    <ul class="task-list">
      {#each tasks as task, index (task.id)}
        <TaskItem
          {task}
          current={app.snapshot!.timer.taskId === task.id}
          first={index === 0}
          last={index === tasks.length - 1}
          toggle={() =>
            app.run(() => api.task({ action: 'toggle', id: task.id }))}
          edit={(title) =>
            app.run(() => api.task({ action: 'update', id: task.id, title }))}
          remove={() =>
            app.run(() => api.task({ action: 'delete', id: task.id }))}
          associate={() =>
            app.run(() =>
              api.task({
                action: 'associate',
                id: app.snapshot!.timer.taskId === task.id ? null : task.id,
              }),
            )}
          move={(offset) => move(index, offset)}
        />
      {:else}<li class="empty-state">
          <span>Make room for one thing.</span>
          <p>Add a task and give it your attention.</p>
        </li>{/each}
    </ul>
    <form
      class="task-input task-composer"
      onsubmit={(e) => {
        e.preventDefault();
        void add();
      }}
    >
      <Plus size={15} /><input
        aria-label="New task"
        bind:value={title}
        placeholder="Add a task…"
        disabled={app.busy}
        maxlength="200"
        required
      /><button
        class="text-button"
        type="submit"
        disabled={app.busy || !title.trim()}
        >{app.busy ? 'Saving…' : 'Add'}</button
      >
    </form>
  {/if}
</section>
