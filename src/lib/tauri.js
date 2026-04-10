import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function listAccounts() {
  return await invoke("list_accounts");
}

export async function addAccount(input) {
  return await invoke("add_account", { input });
}

export async function deleteAccounts(ids) {
  return await invoke("delete_accounts", { ids });
}

export async function importAccounts(accounts) {
  return await invoke("import_accounts", { accounts });
}

export async function exportAccounts() {
  return await invoke("export_accounts");
}

export async function getAccountStats() {
  return await invoke("get_account_stats");
}

export async function switchAccount(accountId) {
  return await invoke("switch_account", { accountId });
}

export async function getAccountToken(accountId) {
  return await invoke("get_account_token", { accountId });
}

export async function refreshPlanStatus(accountId) {
  return await invoke("refresh_plan_status", { accountId });
}

export async function checkWindsurfStatus() {
  return await invoke("check_windsurf_status");
}

export async function getWindsurfPath() {
  return await invoke("get_windsurf_path");
}

export async function setWindsurfPath(path) {
  return await invoke("set_windsurf_path", { path });
}

export async function getActiveAccount() {
  return await invoke("get_active_account");
}

export async function getSetting(key) {
  return await invoke("get_setting", { key });
}

export async function setSetting(key, value) {
  return await invoke("set_setting", { key, value });
}

export async function createGroup(name) {
  return await invoke("create_group", { name });
}

export async function listGroups() {
  return await invoke("list_groups");
}

export async function setAccountsGroup(ids, groupName) {
  return await invoke("set_accounts_group", { ids, groupName });
}

export async function renameGroup(oldName, newName) {
  return await invoke("rename_group", { oldName, newName });
}

export async function deleteGroup(groupName) {
  return await invoke("delete_group", { groupName });
}

export async function checkUpdate() {
  return await invoke("check_update");
}

export function onSwitchProgress(callback) {
  return listen("switch-progress", (event) => {
    callback(event.payload);
  });
}
