<script lang="ts">
  // Main screen: sidebar (folders/tags), searchable list, entry details.
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { api, type Vault, type Entry, type Folder } from "./api";

  let { onLock }: { onLock: () => void } = $props();

  const ROOT = "00000000-0000-0000-0000-000000000000";
  const IDLE_LOCK_MS = 5 * 60 * 1000;

  let vault = $state<Vault | null>(null);
  let search = $state("");
  // What the sidebar filters on: everything, favorites, one folder, or one tag.
  let filter = $state<{ kind: "all" | "fav" | "folder" | "tag"; id?: string }>({ kind: "all" });
  let selectedId = $state<string | null>(null);
  let revealed = $state<string | null>(null);
  let syncing = $state(false);
  let syncError = $state("");
  let toast = $state("");
  let now = $state(Date.now());
  let searchInput = $state<HTMLInputElement>();

  // $derived values recompute automatically when what they read changes.
  let tagsById = $derived(new Map((vault?.tags ?? []).map((t) => [t.id, t])));
  let foldersById = $derived(new Map((vault?.folders ?? []).map((f) => [f.id, f])));
  let folderTree = $derived(buildTree(vault?.folders ?? []));
  let selected = $derived(vault?.entries.find((e) => e.id === selectedId) ?? null);

  let visible = $derived.by(() => {
    if (!vault) return [];
    const q = search.trim().toLowerCase();
    return vault.entries
      .filter((e) => {
        // Searching looks through everything, ignoring the sidebar filter.
        if (q) return [e.label, e.username, e.url, e.notes].some((s) => s.toLowerCase().includes(q));
        if (filter.kind === "fav") return e.favorite;
        if (filter.kind === "folder") return e.folder === filter.id;
        if (filter.kind === "tag") return e.tags.includes(filter.id!);
        return true;
      })
      .sort((a, b) => a.label.localeCompare(b.label));
  });

  /** Flatten folders into a list with depth, parents before children. */
  function buildTree(folders: Folder[]) {
    const out: { folder: Folder; depth: number }[] = [];
    const walk = (parent: string, depth: number) => {
      folders
        .filter((f) => f.parent === parent)
        .sort((a, b) => a.label.localeCompare(b.label))
        .forEach((f) => {
          out.push({ folder: f, depth });
          walk(f.id, depth + 1);
        });
    };
    walk(ROOT, 0);
    return out;
  }

  function customFields(e: Entry): { label: string; type: string; value: string }[] {
    try {
      return JSON.parse(e.customFields || "[]");
    } catch {
      return [];
    }
  }

  function ago(ts: number | null | undefined) {
    if (!ts) return "never";
    const s = Math.max(0, Math.floor(now / 1000 - ts));
    if (s < 60) return "just now";
    if (s < 3600) return `${Math.floor(s / 60)} min ago`;
    if (s < 86400) return `${Math.floor(s / 3600)} h ago`;
    return new Date(ts * 1000).toLocaleString();
  }

  function flash(msg: string) {
    toast = msg;
    setTimeout(() => (toast = ""), 2500);
  }

  async function copy(which: "password" | "username" | "url") {
    if (!selected) return;
    await api.copy(selected.id, which);
    flash(which === "password" ? "Password copied — clears in 20 s" : `${which} copied`);
  }

  async function toggleReveal() {
    revealed = revealed === null && selected ? await api.reveal(selected.id) : null;
  }

  function select(id: string) {
    selectedId = id;
    revealed = null;
  }

  async function sync() {
    syncing = true;
    syncError = "";
    try {
      await api.sync();
      vault = await api.getVault();
    } catch (err) {
      syncError = String(err);
    } finally {
      syncing = false;
    }
  }

  /**
   * Global shortcuts. Ctrl on Linux/Windows, Cmd (metaKey) on macOS.
   * Ctrl+C only copies the password when nothing is selected on the page and
   * the focus isn't in a text field, so normal text copying keeps working.
   */
  function onKeydown(ev: KeyboardEvent) {
    const mod = ev.ctrlKey || ev.metaKey;
    const key = ev.key.toLowerCase();
    if (mod && key === "f") {
      ev.preventDefault();
      searchInput?.focus();
      searchInput?.select();
    } else if (mod && key === "l") {
      ev.preventDefault();
      onLock();
    } else if (mod && key === "c") {
      const typing = ev.target instanceof HTMLInputElement || ev.target instanceof HTMLTextAreaElement;
      const hasSelection = (window.getSelection()?.toString() ?? "") !== "";
      if (selected && !typing && !hasSelection) {
        ev.preventDefault();
        copy("password");
      }
    } else if (key === "escape" && document.activeElement === searchInput) {
      search = "";
      searchInput?.blur();
    }
  }

  onMount(() => {
    api.getVault().then((v) => {
      vault = v;
      sync(); // try to refresh; if offline we keep the local copy
    });

    // Auto-lock after inactivity.
    let last = Date.now();
    const bump = () => (last = Date.now());
    const events = ["mousemove", "keydown", "mousedown", "wheel"];
    events.forEach((ev) => window.addEventListener(ev, bump));
    const timer = setInterval(() => {
      now = Date.now();
      if (now - last > IDLE_LOCK_MS) onLock();
    }, 10_000);

    // Whatever onMount returns runs when the component goes away.
    return () => {
      clearInterval(timer);
      events.forEach((ev) => window.removeEventListener(ev, bump));
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if vault}
  <div class="layout">
    <aside>
      <button class:active={filter.kind === "all"} onclick={() => (filter = { kind: "all" })}>
        All <span>{vault.entries.length}</span>
      </button>
      <button class:active={filter.kind === "fav"} onclick={() => (filter = { kind: "fav" })}>★ Favorites</button>

      <h3>Folders</h3>
      {#each folderTree as { folder, depth } (folder.id)}
        <button
          class:active={filter.kind === "folder" && filter.id === folder.id}
          style="padding-left: {12 + depth * 14}px"
          onclick={() => (filter = { kind: "folder", id: folder.id })}>📁 {folder.label}</button
        >
      {/each}
      <button
        class:active={filter.kind === "folder" && filter.id === ROOT}
        onclick={() => (filter = { kind: "folder", id: ROOT })}>📁 (no folder)</button
      >

      <h3>Tags</h3>
      {#each vault.tags as tag (tag.id)}
        <button
          class:active={filter.kind === "tag" && filter.id === tag.id}
          onclick={() => (filter = { kind: "tag", id: tag.id })}
          ><span class="dot" style="background: {tag.color}"></span> {tag.label}</button
        >
      {/each}

      <div class="footer">
        <div class="muted" title={vault.server}>{vault.user}</div>
        <div class="muted">Synced {ago(vault.syncedAt)}</div>
        {#if syncError}<div class="error" title={syncError}>Offline: using local copy</div>{/if}
        <div class="row">
          <button onclick={sync} disabled={syncing}>{syncing ? "Syncing…" : "⟳ Sync"}</button>
          <button onclick={onLock} title="Ctrl+L">🔒 Lock</button>
        </div>
      </div>
    </aside>

    <section class="list">
      <input
        class="search"
        placeholder="Search {vault.entries.length} passwords…  (Ctrl+F)"
        bind:this={searchInput}
        bind:value={search}
      />
      <div class="items">
        {#each visible as e (e.id)}
          <button class="item" class:active={e.id === selectedId} onclick={() => select(e.id)}>
            <div class="label">{e.favorite ? "★ " : ""}{e.label}</div>
            <div class="muted">{e.username}</div>
          </button>
        {:else}
          <p class="muted empty">Nothing here.</p>
        {/each}
      </div>
    </section>

    <section class="detail">
      {#if selected}
        <h2>{selected.label}</h2>
        <dl>
          <dt>Username</dt>
          <dd>
            <code>{selected.username || "—"}</code>
            {#if selected.username}<button onclick={() => copy("username")}>Copy</button>{/if}
          </dd>

          <dt>Password</dt>
          <dd>
            <code>{revealed ?? "••••••••••••"}</code>
            <button onclick={toggleReveal}>{revealed === null ? "Show" : "Hide"}</button>
            <button class="primary" onclick={() => copy("password")} title="Ctrl+C">Copy</button>
          </dd>

          {#if selected.url}
            <dt>URL</dt>
            <dd>
              <code class="url">{selected.url}</code>
              <button onclick={() => copy("url")}>Copy</button>
              <button onclick={() => openUrl(selected!.url)}>Open</button>
            </dd>
          {/if}

          {#if selected.folder !== ROOT}
            <dt>Folder</dt>
            <dd>{foldersById.get(selected.folder)?.label ?? "?"}</dd>
          {/if}

          {#if selected.tags.length}
            <dt>Tags</dt>
            <dd>
              {#each selected.tags as id}
                {@const t = tagsById.get(id)}
                {#if t}<span class="tag" style="border-color: {t.color}">{t.label}</span>{/if}
              {/each}
            </dd>
          {/if}

          {#each customFields(selected) as f}
            <dt>{f.label}</dt>
            <dd><code>{f.type === "secret" ? "••••••" : f.value}</code></dd>
          {/each}

          {#if selected.notes}
            <dt>Notes</dt>
            <dd><pre>{selected.notes}</pre></dd>
          {/if}

          <dt>Last edited</dt>
          <dd class="muted">{new Date(selected.edited * 1000).toLocaleString()}</dd>
        </dl>
      {:else}
        <p class="muted empty">Select an entry.</p>
      {/if}
    </section>
  </div>
  {#if toast}<div class="toast">{toast}</div>{/if}
{/if}

<style>
  .layout {
    display: grid;
    grid-template-columns: 220px 300px 1fr;
    height: 100vh;
  }
  aside {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 6px;
    border-right: 1px solid var(--border);
    overflow-y: auto;
  }
  aside > button {
    text-align: left;
    border: none;
    background: none;
    padding: 5px 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: flex;
    gap: 6px;
    align-items: center;
  }
  aside > button span {
    margin-left: auto;
    color: var(--muted);
  }
  aside button.active,
  .item.active {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
  }
  h3 {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--muted);
    margin: 14px 12px 4px;
  }
  .dot {
    display: inline-block;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    margin-left: 0 !important;
  }
  .footer {
    margin-top: auto;
    padding: 10px 6px 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
  }
  .row {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }
  .list {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
    min-height: 0;
  }
  .search {
    margin: 10px;
  }
  .items {
    overflow-y: auto;
    flex: 1;
  }
  .item {
    display: block;
    width: 100%;
    text-align: left;
    border: none;
    border-radius: 0;
    border-bottom: 1px solid var(--border);
    background: none;
    padding: 8px 12px;
  }
  .label {
    font-weight: 600;
  }
  .muted {
    color: var(--muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .detail {
    padding: 20px 24px;
    overflow-y: auto;
  }
  h2 {
    margin-top: 0;
  }
  dt {
    font-size: 11px;
    text-transform: uppercase;
    color: var(--muted);
    margin-top: 14px;
  }
  dd {
    margin: 4px 0 0;
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
  }
  code {
    font-size: 14px;
    word-break: break-all;
  }
  pre {
    white-space: pre-wrap;
    margin: 0;
    font-family: inherit;
  }
  .tag {
    border: 2px solid;
    border-radius: 10px;
    padding: 0 8px;
  }
  .empty {
    padding: 20px;
    text-align: center;
  }
  .toast {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--text);
    color: var(--bg);
    padding: 8px 16px;
    border-radius: 6px;
  }
</style>
