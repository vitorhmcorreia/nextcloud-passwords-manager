# Learning Svelte (and a bit of TypeScript) with ncpass

The UI lives in `src/`. Svelte is a compiler: you write `.svelte` components, and at build
time they're turned into small, fast JavaScript. No virtual DOM.

Resources:
- [Interactive Svelte tutorial](https://svelte.dev/tutorial). Do this first; it takes ~2 hours.
- [Svelte docs](https://svelte.dev/docs/svelte)
- [TypeScript in 5 minutes](https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes.html)

We use **Svelte 5** ("runes" syntax: `$state`, `$derived`, `$props`). Older tutorials
use `let x` + `$:` instead; the ideas are the same.

---

## 1. Anatomy of a component

```svelte
<script lang="ts">
  // TypeScript: state and logic
</script>

<!-- HTML with {expressions} -->

<style>
  /* CSS, automatically scoped to this component only */
</style>
```

Scoped CSS means `.card` in `Setup.svelte` doesn't affect `.card` in `Unlock.svelte`.
Use `:global(...)` for styles that should apply everywhere (see `+page.svelte`).

## 2. TypeScript basics you'll see

```ts
export type Entry = { id: string; label: string; tags: string[]; favorite: boolean };
let screen = $state<Screen>("loading");       // generic type parameter
type Screen = "loading" | "setup" | "locked" | "open";  // union of literal strings
vault?.tags ?? []     // ?. = stop if null;  ?? = fallback if null/undefined
```

Types only exist at compile time, and `pnpm check` verifies them. `src/lib/api.ts` mirrors
the Rust structs, so if you add a field in Rust, add it there too.

## 3. Reactive state: `$state`

```ts
let search = $state("");
```

When `search` changes, every part of the HTML that uses it updates. You just
assign: `search = "abc"`. There's no `setState`.

## 4. Computed values: `$derived`

```ts
let selected = $derived(vault?.entries.find((e) => e.id === selectedId) ?? null);
let visible = $derived.by(() => { /* longer code */ return list; });
```

These recompute automatically when anything they read (`vault`, `selectedId`, `search`, …)
changes. The whole search + sidebar filter in `VaultView.svelte` is one `$derived.by`.

## 5. Props: `$props`

A parent passes data and callbacks down to a child:

```svelte
<!-- +page.svelte -->
<Unlock onUnlocked={() => (screen = "open")} onReset={() => (screen = "setup")} />
```

```ts
// Unlock.svelte
let { onUnlocked, onReset }: { onUnlocked: () => void; onReset: () => void } = $props();
```

Children call `onUnlocked()` to tell the parent something happened.

## 6. Template syntax

```svelte
{#if error}<p class="error">{error}</p>{/if}

{#if screen === "setup"} ... {:else if screen === "locked"} ... {/if}

{#each visible as e (e.id)}        <!-- (e.id) = key, helps Svelte track rows -->
  <button onclick={() => select(e.id)}>{e.label}</button>
{:else}
  <p>Nothing here.</p>             <!-- shown when the list is empty -->
{/each}

{@const t = tagsById.get(id)}      <!-- local constant inside a block -->
```

- **Events**: `onclick={fn}`, `onsubmit={submit}` (plain HTML attribute names).
- **Two-way binding**: `<input bind:value={search} />` keeps the variable and the input in sync.
- **Conditional class**: `class:active={e.id === selectedId}`.
- **Inline style**: `style="padding-left: {12 + depth * 14}px"`.

## 7. Lifecycle: `onMount`

```ts
onMount(() => {
  // runs once when the component appears
  const timer = setInterval(...);
  return () => clearInterval(timer);   // cleanup when it disappears
});
```

`VaultView` uses this to load the vault, start the first sync and set up the idle auto-lock.

## 8. Talking to Rust

```ts
import { invoke } from "@tauri-apps/api/core";
invoke<string>("reveal", { id })   // → Promise<string>
```

We wrap every command in `src/lib/api.ts` so components call `api.reveal(id)` with types.
Errors from Rust (`Err("...")`) become rejected promises, so use `try { await ... } catch (err)`,
as in `Setup.svelte`.

## 9. SvelteKit bits

The project uses SvelteKit only for routing and building (`adapter-static`, SPA mode).
`src/routes/+page.svelte` is the single page, `src/routes/+layout.ts` turns off server
rendering, and `$lib` is an alias for `src/lib`.

## Exercises

1. Add a count next to each folder in the sidebar (hint: a `$derived` Map of folder → count).
2. Press `Escape` to clear the search box (`onkeydown` on the input).
3. Show the entry's URL domain in the list under the username.
4. Make favorites sort first in the list.
5. Add a "copy" button for custom fields (needs a new Rust command; see Rust exercise 2).
