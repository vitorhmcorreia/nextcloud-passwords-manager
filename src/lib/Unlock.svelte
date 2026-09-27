<script lang="ts">
  // Lock screen: ask for the master password.
  import { api } from "./api";

  let { onUnlocked, onReset }: { onUnlocked: () => void; onReset: () => void } = $props();

  let master = $state("");
  let busy = $state(false);
  let error = $state("");
  let confirmReset = $state(false);

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    busy = true;
    error = "";
    try {
      await api.unlock(master);
      master = "";
      onUnlocked();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function reset() {
    await api.reset();
    onReset();
  }
</script>

<form class="card" onsubmit={submit}>
  <h1>🔒 ncpass</h1>
  <!-- svelte-ignore a11y_autofocus -->
  <input type="password" placeholder="Master password" bind:value={master} autofocus required />
  {#if error}<p class="error">{error}</p>{/if}
  <button class="primary" disabled={busy}>{busy ? "Unlocking…" : "Unlock"}</button>

  {#if confirmReset}
    <p class="error">This deletes the local copy. You'll need to set up again (server data is not touched).</p>
    <div class="row">
      <button type="button" onclick={() => (confirmReset = false)}>Cancel</button>
      <button type="button" class="danger" onclick={reset}>Delete local copy</button>
    </div>
  {:else}
    <button type="button" class="link" onclick={() => (confirmReset = true)}>Forgot master password?</button>
  {/if}
</form>

<style>
  .card {
    max-width: 320px;
    margin: 120px auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 24px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  h1 {
    margin: 0 0 8px;
    font-size: 20px;
    text-align: center;
  }
  .row {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .link {
    border: none;
    background: none;
    color: var(--muted);
    font-size: 12px;
  }
  .danger {
    color: var(--danger);
  }
</style>
