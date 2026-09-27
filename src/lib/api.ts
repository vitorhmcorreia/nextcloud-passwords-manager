// Typed wrappers around the Rust commands in src-tauri/src/commands.rs.
import { invoke } from "@tauri-apps/api/core";

export type Status = { exists: boolean; unlocked: boolean };

export type Entry = {
  id: string;
  label: string;
  username: string;
  url: string;
  notes: string;
  folder: string;
  tags: string[];
  favorite: boolean;
  edited: number;
  customFields: string;
};
export type Folder = { id: string; label: string; parent: string };
export type Tag = { id: string; label: string; color: string };

export type Vault = {
  server: string;
  user: string;
  syncedAt: number | null;
  entries: Entry[];
  folders: Folder[];
  tags: Tag[];
};

export type Settings = { autoLockMins: number; clipboardClearSecs: number };

export const api = {
  status: () => invoke<Status>("status"),
  setup: (server: string, user: string, password: string, master: string) =>
    invoke<void>("setup", { server, user, password, master }),
  unlock: (master: string) => invoke<void>("unlock", { master }),
  lock: () => invoke<void>("lock"),
  reset: () => invoke<void>("reset"),
  getVault: () => invoke<Vault>("get_vault"),
  reveal: (id: string) => invoke<string>("reveal", { id }),
  copy: (id: string, which: "password" | "username" | "url") =>
    invoke<void>("copy", { id, which }),
  sync: () => invoke<number>("sync"),
  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke<void>("set_settings", { settings }),
};
