<script lang="ts">
  import { ArrowLeft, Volume2, Upload } from '@lucide/svelte';
  import { app } from '../../state/app.svelte';
  import { api } from '../../api/commands';
  import { defaultSettings } from '../../types/contracts';
  import IconButton from '../common/IconButton.svelte';
  import Select from '../common/Select.svelte';
  let { close }: { close: () => void } = $props();
  let draft = $state({ ...app.snapshot!.settings });
  let reset = $state(false);
  let saved = $state(false);
  async function save() {
    saved = await app.run(
      () => api.settings({ ...draft }),
      'Routine settings saved.',
    );
  }
  async function preview(kind: 'alarm' | 'tick') {
    await app.run(() => api.preview(kind, { ...draft }));
  }
</script>

<div class="settings-panel">
  <div class="settings-title">
    <IconButton label="Back to timer" onclick={close}
      ><ArrowLeft size={16} /></IconButton
    >
    <h1>Your routine</h1>
  </div>
  <form
    class="settings-form"
    onsubmit={(e) => {
      e.preventDefault();
      void save();
    }}
    oninput={() => (saved = false)}
  >
    <div class="settings-scroll">
      <fieldset>
        <legend>Focus & rest</legend>
        <div class="number-grid">
          <label
            >Focus length <span>minutes</span><input
              type="number"
              min="1"
              max="180"
              step="1"
              bind:value={draft.focusMinutes}
              required
            /></label
          >
          <label
            >Short break <span>minutes</span><input
              type="number"
              min="1"
              max="60"
              step="1"
              bind:value={draft.shortBreakMinutes}
              required
            /></label
          >
          <label
            >Focus sessions <span>per round</span><input
              type="number"
              min="1"
              max="12"
              step="1"
              bind:value={draft.sessionsPerCycle}
              required
            /></label
          >
        </div>
        <p class="hint">Start once. Focus and breaks follow automatically.</p>
        <label class="switch-row"
          >Repeat routine<input
            type="checkbox"
            bind:checked={draft.repeatRoutine}
          /></label
        >
      </fieldset>
      <fieldset>
        <legend>Long break</legend>
        <label class="switch-row"
          >Include long breaks<input
            type="checkbox"
            bind:checked={draft.longBreakEnabled}
          /></label
        >
        {#if draft.longBreakEnabled}
          <div class="number-grid">
            <label
              >Long break length <span>minutes</span><input
                type="number"
                min="1"
                max="120"
                step="1"
                bind:value={draft.longBreakMinutes}
                required
              /></label
            >
            <label
              >Long break every <span>focus sessions</span><input
                type="number"
                min="1"
                max="12"
                step="1"
                bind:value={draft.longBreakInterval}
                required
              /></label
            >
          </div>
          <p class="hint">
            Replaces the short break after every {draft.longBreakInterval} completed
            focus sessions.
          </p>
        {:else}<p class="hint">Short breaks between focus sessions.</p>{/if}
      </fieldset>
      <fieldset>
        <legend>Sound</legend>
        <label class="switch-row"
          >Completion alarm<input
            type="checkbox"
            bind:checked={draft.alarmEnabled}
          /></label
        >
        <div class="select-row">
          <span>Alarm</span><Select
            label="Alarm"
            value={draft.alarm}
            options={[
              { value: 'chime', label: 'Soft chime' },
              { value: 'bell', label: 'Classic bell' },
              { value: 'digital', label: 'Digital notification' },
              ...(app.snapshot!.customAudio
                ? [{ value: 'custom', label: 'Custom sound' }]
                : []),
            ]}
            onchange={(value) => {
              draft.alarm = value;
              saved = false;
            }}
            disabled={app.busy}
          />
        </div>
        <div class="sound-actions">
          <button
            class="text-button"
            type="button"
            onclick={async () => {
              if ((await app.run(api.importAudio)) && app.snapshot!.customAudio)
                draft.alarm = 'custom';
            }}><Upload size={12} />Import sound</button
          ><button
            class="text-button"
            type="button"
            onclick={() => preview('alarm')}
            ><Volume2 size={12} />Preview</button
          >
        </div>
        <label class="range-row"
          >Alarm volume <span>{Math.round(draft.alarmVolume * 100)}%</span
          ><input
            aria-label="Alarm volume"
            type="range"
            min="0"
            max="1"
            step=".01"
            bind:value={draft.alarmVolume}
          /></label
        >
        <label class="switch-row"
          >Ticking sound<input
            type="checkbox"
            bind:checked={draft.tickEnabled}
          /></label
        >
        <div class="select-row">
          <span>Tick</span><Select
            label="Tick"
            value={draft.tick}
            options={[
              { value: 'mechanical', label: 'Mechanical' },
              { value: 'soft', label: 'Soft digital' },
            ]}
            onchange={(value) => {
              draft.tick = value;
              saved = false;
            }}
            disabled={app.busy}
          />
        </div>
        <label class="range-row"
          >Tick volume <span>{Math.round(draft.tickVolume * 100)}%</span><input
            aria-label="Tick volume"
            type="range"
            min="0"
            max="1"
            step=".01"
            bind:value={draft.tickVolume}
          /></label
        >
        <button
          class="text-button"
          type="button"
          onclick={() => preview('tick')}
          ><Volume2 size={12} />Preview tick</button
        >
      </fieldset>
      <fieldset>
        <legend>Preferences</legend>
        <label class="switch-row"
          >Windows notifications<input
            type="checkbox"
            bind:checked={draft.notifications}
          /></label
        >
        <label class="switch-row"
          >Reduced motion<input
            type="checkbox"
            bind:checked={draft.reducedMotion}
          /></label
        >
        <label class="switch-row"
          >Always on top<input
            type="checkbox"
            bind:checked={draft.alwaysOnTop}
          /></label
        >
        <label class="switch-row"
          >Start with Windows<input
            type="checkbox"
            bind:checked={draft.startWithWindows}
          /></label
        >
        <label class="switch-row"
          >Remember position<input
            type="checkbox"
            bind:checked={draft.rememberPosition}
          /></label
        >
        <label class="switch-row"
          >Close to tray<input
            type="checkbox"
            bind:checked={draft.closeToTray}
          /></label
        >
        {#if reset}<div class="confirmation">
            <p>Restore default preferences? Tasks are kept.</p>
            <button
              class="text-button"
              type="button"
              onclick={() => {
                draft = { ...defaultSettings };
                reset = false;
                saved = false;
              }}>Restore defaults</button
            ><button
              class="text-button"
              type="button"
              onclick={() => (reset = false)}>Cancel</button
            >
          </div>{:else}<button
            class="text-button"
            type="button"
            onclick={() => (reset = true)}>Reset preferences</button
          >{/if}
        <button
          class="text-button"
          type="button"
          onclick={() => app.run(api.quit)}>Quit RVNFlow</button
        >
      </fieldset>
      <p class="version">RVNFlow · A little space to focus</p>
    </div>
    <div class="settings-save">
      <span aria-live="polite"
        >{saved ? 'Saved' : 'Make time for your best work.'}</span
      ><button class="primary-button" type="submit" disabled={app.busy}
        >Save changes</button
      >
    </div>
  </form>
</div>
