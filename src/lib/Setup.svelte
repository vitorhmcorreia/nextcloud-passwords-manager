<script lang="ts">
  // First run: ask for the Nextcloud login and a new local master password.
  import { api } from "./api";

  let { onDone }: { onDone: () => void } = $props();

  let server = $state("https://");
  let user = $state("");
  let password = $state("");
  let master = $state("");
  let master2 = $state("");
  let busy = $state(false);
  let error = $state("");

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    error = "";
    if (master !== master2) {
      error = "Master passwords do not match";
      return;
    }
    busy = true;
    try {
      await api.setup(server, user, password, master);
      onDone();
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }
</script>

<form class="card" onsubmit={submit}>
  <h1>Set up ncpass</h1>
  <p class="muted">Connects to your Nextcloud Passwords and keeps an encrypted copy on this computer.</p>

  <label>Nextcloud URL <input bind:value={server} required /></label>
  <label>User <input bind:value={user} required autocomplete="username" /></label>
  <label>Nextcloud password <input type="password" bind:value={password} required /></label>

  <hr />
  <p class="muted">
    The master password encrypts the local copy. It never leaves this computer and
    <strong>cannot be recovered</strong>. If you forget it, just set up again.
  </p>
  <label>Master password <input type="password" bind:value={master} required minlength="8" /></label>
  <label>Repeat master password <input type="password" bind:value={master2} required /></label>

  {#if error}<p class="error">{error}</p>{/if}
  <button class="primary" disabled={busy}>{busy ? "Downloading…" : "Connect and download"}</button>
</form>

<style>
  .card {
    max-width: 420px;
    margin: 40px auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 24px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  h1 {
    margin: 0;
    font-size: 20px;
  }
  hr {
    border: none;
    border-top: 1px solid var(--border);
    width: 100%;
  }
  .muted {
    color: var(--muted);
    margin: 0;
  }
</style>
