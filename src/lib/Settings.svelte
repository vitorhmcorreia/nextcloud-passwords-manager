<script lang="ts">
  // Settings dialog: auto-lock and clipboard timings. Saved as settings.json by Rust.
  import { onMount } from "svelte";
  import { api, type Settings } from "./api";

  let { current, onSaved, onClose }: { current: Settings; onSaved: (s: Settings) => void; onClose: () => void } =
    $props();

  const LOCK_MINS = [1, 2, 5, 10, 15, 30, 60];
  const CLEAR_SECS = [10, 20, 30, 45, 60, 90, 120];

  // Edit a copy, so Cancel leaves the real settings alone.
  // svelte-ignore state_referenced_locally
  let draft = $state({ ...current });
  let error = $state("");
  let dialog: HTMLDialogElement;

  // showModal() gives us a backdrop, focus trapping and Esc-to-close for free.
  onMount(() => dialog.showModal());

  async function save(e: SubmitEvent) {
    e.preventDefault();
    error = "";
    try {
      await api.setSettings(draft);
      onSaved(draft);
    } catch (err) {
      error = String(err);
    }
  }
</script>

<dialog bind:this={dialog} onclose={onClose}>
  <form onsubmit={save}>
    <h2>Settings</h2>

    <label>
      Lock after being idle for
      <select bind:value={draft.autoLockMins}>
        {#each LOCK_MINS as m}<option value={m}>{m} min</option>{/each}
      </select>
    </label>

    <label>
      Clear copied passwords after
      <select bind:value={draft.clipboardClearSecs}>
        {#each CLEAR_SECS as s}<option value={s}>{s} s</option>{/each}
      </select>
    </label>

    {#if error}<p class="error">{error}</p>{/if}

    <div class="row">
      <button type="button" onclick={() => dialog.close()}>Cancel</button>
      <button class="primary">Save</button>
    </div>
  </form>
</dialog>

<style>
  dialog {
    background: var(--panel);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 20px 24px;
    width: 320px;
  }
  dialog::backdrop {
    background: rgb(0 0 0 / 0.35);
  }
  h2 {
    margin: 0 0 16px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 14px;
  }
  select {
    font: inherit;
    color: inherit;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 8px;
  }
  /* Fallback for platforms that ignore color-scheme on the open list. */
  option {
    background: var(--panel);
    color: var(--text);
  }
  .row {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
