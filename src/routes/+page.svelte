<script lang="ts">
  // Top-level screen switcher: first-run setup, lock screen, or the vault.
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import Setup from "$lib/Setup.svelte";
  import Unlock from "$lib/Unlock.svelte";
  import VaultView from "$lib/VaultView.svelte";

  type Screen = "loading" | "setup" | "locked" | "open";
  let screen = $state<Screen>("loading");

  async function refresh() {
    const s = await api.status();
    screen = s.unlocked ? "open" : s.exists ? "locked" : "setup";
  }

  async function lock() {
    await api.lock();
    screen = "locked";
  }

  onMount(refresh);
</script>

<main>
  {#if screen === "setup"}
    <Setup onDone={() => (screen = "open")} />
  {:else if screen === "locked"}
    <Unlock onUnlocked={() => (screen = "open")} onReset={() => (screen = "setup")} />
  {:else if screen === "open"}
    <VaultView onLock={lock} />
  {/if}
</main>

<style>
  :global(:root) {
    --bg: #f6f7f9;
    --panel: #ffffff;
    --text: #1d2330;
    --muted: #6b7280;
    --border: #dfe3ea;
    --accent: #0082c9; /* Nextcloud blue */
    --danger: #c62828;
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
    font-size: 14px;
    color: var(--text);
    background: var(--bg);
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #16181d;
      --panel: #1f2229;
      --text: #e6e8ec;
      --muted: #9aa1ad;
      --border: #333844;
    }
  }
  :global(body) {
    margin: 0;
  }
  :global(button),
  :global(input) {
    font: inherit;
    color: inherit;
  }
  :global(button) {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 12px;
    cursor: pointer;
  }
  :global(button.primary) {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }
  :global(button:disabled) {
    opacity: 0.6;
    cursor: default;
  }
  :global(input) {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 7px 10px;
  }
  :global(.error) {
    color: var(--danger);
  }
  main {
    height: 100vh;
  }
</style>
