<script lang="ts">
  import { ArrowLeft, Volume2, Upload, Check } from '@lucide/svelte';
  import { app } from '../../state/app.svelte';
  import { api } from '../../api/commands';
  import { defaultSettings } from '../../types/contracts';
  import IconButton from '../common/IconButton.svelte';
  let { close }: { close: () => void } = $props();
  let draft = $state({ ...app.snapshot!.settings });
  let reset = $state(false);
  let saved = $state(false);
  async function save() {
    const success = await app.run(
      () => api.settings({ ...draft }),
      'Settings saved.',
    );
    saved = success;
    return success;
  }
  async function preview(kind: 'alarm' | 'tick') {
    await app.run(() => api.preview(kind, { ...draft }));
  }
</script>

<div class="settings-panel">
  <div class="settings-title">
    <IconButton label="Back to timer" onclick={close}
      ><ArrowLeft size={17} /></IconButton
    >
    <h1>Your rhythm</h1>
    <span>Settings</span>
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
        <legend>Timer</legend>
        <div class="number-grid">
          <label
            >Focus <span>minutes</span><input
              type="number"
              min="1"
              max="180"
              step="1"
              bind:value={draft.focusMinutes}
              required
            /></label
          ><label
            >Short break <span>minutes</span><input
              type="number"
              min="1"
              max="60"
              step="1"
              bind:value={draft.shortBreakMinutes}
              required
            /></label
          ><label
            >Long break <span>minutes</span><input
              type="number"
              min="1"
              max="120"
              step="1"
              bind:value={draft.longBreakMinutes}
              required
            /></label
          ><label
            >Sessions <span>per cycle</span><input
              type="number"
              min="1"
              max="12"
              step="1"
              bind:value={draft.sessionsPerCycle}
              required
            /></label
          >
        </div>
        <p class="hint">
          New durations apply to the next session. Your running timer keeps its
          time.
        </p>
      </fieldset>
      <fieldset>
        <legend>Session automation</legend>
        <label class="switch-row"
          >Auto-start breaks<input
            type="checkbox"
            bind:checked={draft.autoStartBreaks}
          /></label
        >
        <label class="switch-row"
          >Auto-start focus sessions<input
            type="checkbox"
            bind:checked={draft.autoStartFocus}
          /></label
        >
        <p class="hint">
          Start the next session automatically. Both options are independent.
        </p>
      </fieldset>
      <fieldset>
        <legend>Appearance</legend>
        <div class="theme-chip">
          <span></span>Lavender Dark <Check size={13} />
        </div>
        <label class="switch-row"
          >Reduced motion<input
            type="checkbox"
            bind:checked={draft.reducedMotion}
          /></label
        ><label class="switch-row"
          >Always on top<input
            type="checkbox"
            bind:checked={draft.alwaysOnTop}
          /></label
        >
      </fieldset>
      <fieldset>
        <legend>Sound</legend><label class="switch-row"
          >Completion alarm<input
            type="checkbox"
            bind:checked={draft.alarmEnabled}
          /></label
        ><label class="select-row"
          >Alarm<select bind:value={draft.alarm}
            ><option value="chime">Soft chime</option><option value="bell"
              >Classic bell</option
            ><option value="digital">Digital notification</option
            >{#if app.snapshot!.customAudio}<option value="custom"
                >Custom sound</option
              >{/if}</select
          ></label
        >
        <div class="sound-actions">
          <button
            class="text-button"
            type="button"
            onclick={async () => {
              if (await app.run(api.importAudio))
                if (app.snapshot!.customAudio) draft.alarm = 'custom';
            }}><Upload size={13} />Import WAV / MP3</button
          ><button
            class="text-button"
            type="button"
            onclick={() => preview('alarm')}
            ><Volume2 size={13} />Preview</button
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
        ><label class="select-row"
          >Tick<select bind:value={draft.tick}
            ><option value="mechanical">Mechanical</option><option value="soft"
              >Soft digital</option
            ></select
          ></label
        ><label class="range-row"
          >Tick volume <span>{Math.round(draft.tickVolume * 100)}%</span><input
            aria-label="Tick volume"
            type="range"
            min="0"
            max="1"
            step=".01"
            bind:value={draft.tickVolume}
          /></label
        ><button
          class="text-button"
          type="button"
          onclick={() => preview('tick')}
          ><Volume2 size={13} />Preview tick</button
        >
      </fieldset>
      <fieldset>
        <legend>Notifications</legend><label class="switch-row"
          >Windows notifications<input
            type="checkbox"
            bind:checked={draft.notifications}
          /></label
        >
      </fieldset>
      <fieldset>
        <legend>Window behavior</legend><label class="switch-row"
          >Start with Windows<input
            type="checkbox"
            bind:checked={draft.startWithWindows}
          /></label
        ><label class="switch-row"
          >Remember position<input
            type="checkbox"
            bind:checked={draft.rememberPosition}
          /></label
        ><label class="switch-row"
          >Close to tray<input
            type="checkbox"
            bind:checked={draft.closeToTray}
          /></label
        >
        <p class="hint">Quit from the tray menu to fully exit RVNFlow.</p>
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
      </fieldset>
      <button
        class="text-button"
        type="button"
        onclick={() => app.run(api.quit)}>Quit RVNFlow</button
      >
      <p class="version">RVNFlow 0.2.0 · Local. Quiet. Yours.</p>
    </div>
    <div class="settings-save">
      <span aria-live="polite"
        >{saved ? 'Preferences saved' : 'Make it your own.'}</span
      ><button class="primary-button" type="submit" disabled={app.busy}
        >Save changes</button
      >
    </div>
  </form>
</div>
