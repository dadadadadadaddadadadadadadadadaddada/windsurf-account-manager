<script setup>
import { ref, watch, onMounted } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import AccountTable from "./components/AccountTable.vue";
import AddAccountDialog from "./components/AddAccountDialog.vue";
import SwitchProgressDialog from "./components/SwitchProgressDialog.vue";
import SettingsDialog from "./components/SettingsDialog.vue";
import UpdateDialog from "./components/UpdateDialog.vue";
import SetGroupDialog from "./components/SetGroupDialog.vue";
import GroupManagerDialog from "./components/GroupManagerDialog.vue";
import {
  listAccounts,
  addAccount,
  deleteAccounts,
  switchAccount,
  getAccountToken,
  refreshPlanStatus,
  checkUpdate,
  getActiveAccount,
  getSetting,
  listGroups,
  onSwitchProgress,
} from "./lib/tauri";

const accounts = ref([]);
const stats = ref({ total: 0, free: 0, trial: 0, pro: 0, unknown: 0 });
const searchQuery = ref("");
const filterType = ref("all");
const selectedIds = ref([]);
const showAddDialog = ref(false);

const switching = ref(false);
const switchProgress = ref(null);

const refreshingIds = ref(new Set());
const batchRefreshing = ref(false);
const batchRefreshProgress = ref({ done: 0, total: 0, failed: 0 });
const confirmDialog = ref(null);
const showSettings = ref(false);
const updateInfo = ref(null);
const activeEmail = ref(null);
const enableGroups = ref(false);
const groups = ref([]);
const filterGroup = ref("all");
const showSetGroup = ref(false);
const showGroupManager = ref(false);

async function loadGroups() {
  try { groups.value = await listGroups(); } catch (_) {}
}

async function loadAccounts() {
  try {
    accounts.value = await listAccounts();
    updateStats();
  } catch (e) {
    console.error("Failed to load accounts:", e);
  }
}

function updateStats() {
  const all = accounts.value;
  stats.value = {
    total: all.length,
    free: all.filter((a) => a.account_type === "Free").length,
    trial: all.filter((a) => a.account_type === "Trial").length,
    pro: all.filter((a) => a.account_type === "Pro").length,
    unknown: all.filter(
      (a) => !["Free", "Trial", "Pro"].includes(a.account_type)
    ).length,
  };
}

const filteredAccounts = ref([]);
function updateFilteredAccounts() {
  let result = accounts.value;
  if (filterType.value !== "all") {
    result = result.filter(
      (a) => a.account_type.toLowerCase() === filterType.value.toLowerCase()
    );
  }
  if (enableGroups.value && filterGroup.value !== "all") {
    if (filterGroup.value === "__ungrouped__") {
      result = result.filter((a) => !a.group_name);
    } else {
      result = result.filter((a) => a.group_name === filterGroup.value);
    }
  }
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter((a) => a.email.toLowerCase().includes(q));
  }
  filteredAccounts.value = result;
}

watch([accounts, searchQuery, filterType, filterGroup, enableGroups], updateFilteredAccounts, {
  immediate: true,
});

async function handleAddAccount(input) {
  try {
    await addAccount(input);
    showAddDialog.value = false;
    await loadAccounts();
  } catch (e) {
    console.error("添加失败:", e);
  }
}

function handleDeleteSelected() {
  if (selectedIds.value.length === 0) return;
  confirmDialog.value = {
    title: '确认删除',
    message: `确定要删除选中的 ${selectedIds.value.length} 个账号吗？此操作不可撤销。`,
    onConfirm: async () => {
      confirmDialog.value = null;
      try {
        await deleteAccounts(selectedIds.value);
        selectedIds.value = [];
        await loadAccounts();
      } catch (e) {
        confirmDialog.value = { title: '删除失败', message: String(e), onConfirm: () => { confirmDialog.value = null; } };
      }
    },
  };
}

async function refreshSingleAccount(id) {
  try {
    await refreshPlanStatus(id);
    return;
  } catch (_) {}
  await getAccountToken(id);
  await refreshPlanStatus(id);
}

async function handleRefreshSingle(accountId) {
  refreshingIds.value.add(accountId);
  try {
    await refreshSingleAccount(accountId);
    await loadAccounts();
  } catch (e) {
    confirmDialog.value = { title: '刷新失败', message: String(e), onConfirm: () => { confirmDialog.value = null; } };
  } finally {
    refreshingIds.value.delete(accountId);
  }
}

async function handleBatchRefresh() {
  const ids = selectedIds.value.length > 0
    ? [...selectedIds.value]
    : accounts.value.map((a) => a.id);
  if (ids.length === 0) return;

  batchRefreshing.value = true;
  batchRefreshProgress.value = { done: 0, total: ids.length, failed: 0 };
  const failures = [];

  const concurrency = 100;
  for (let i = 0; i < ids.length; i += concurrency) {
    const batch = ids.slice(i, i + concurrency);
    const idToEmail = {};
    for (const id of batch) {
      const acc = accounts.value.find((a) => a.id === id);
      if (acc) idToEmail[id] = acc.email;
    }
    const results = await Promise.allSettled(
      batch.map((id) => refreshSingleAccount(id))
    );
    for (let j = 0; j < results.length; j++) {
      batchRefreshProgress.value.done++;
      if (results[j].status === "rejected") {
        batchRefreshProgress.value.failed++;
        failures.push(`${idToEmail[batch[j]] || batch[j]}: ${results[j].reason}`);
      }
    }
  }

  await loadAccounts();
  batchRefreshing.value = false;

  const success = ids.length - failures.length;
  let msg = `成功 ${success}/${ids.length}`;
  if (failures.length > 0) {
    msg += `\n\n失败账号:\n${failures.join('\n')}`;
  }
  confirmDialog.value = {
    title: '刷新配额完成',
    message: msg,
    onConfirm: () => { confirmDialog.value = null; },
  };
}

async function handleSwitch(accountId) {
  switching.value = true;
  switchProgress.value = { step: 0, total: 5, message: "准备切换..." };
  try {
    await switchAccount(accountId);
    switchProgress.value = { step: 5, total: 5, message: "切换完成!" };
    const acc = accounts.value.find((a) => a.id === accountId);
    if (acc) activeEmail.value = acc.email;
    setTimeout(() => {
      switching.value = false;
      switchProgress.value = null;
    }, 1500);
  } catch (e) {
    const errMsg = String(e);
    switchProgress.value = { step: 0, total: 5, message: "切换失败: " + errMsg };
    const isPathError = errMsg.includes("未找到 Windsurf") || errMsg.includes("手动配置安装路径");
    setTimeout(() => {
      switching.value = false;
      switchProgress.value = null;
      if (isPathError) {
        showSettings.value = true;
      }
    }, isPathError ? 1500 : 3000);
  }
}

async function handleCheckUpdate(manual = false) {
  try {
    const info = await checkUpdate();
    if (info.has_update) {
      updateInfo.value = info;
    } else if (manual) {
      confirmDialog.value = {
        title: '检查更新',
        message: `当前已是最新版本 (v${info.current_version})`,
        onConfirm: () => { confirmDialog.value = null; },
      };
    }
  } catch (e) {
    if (manual) {
      confirmDialog.value = {
        title: '检查更新失败',
        message: String(e),
        onConfirm: () => { confirmDialog.value = null; },
      };
    }
  }
}

onMounted(async () => {
  await loadAccounts();
  onSwitchProgress((progress) => {
    switchProgress.value = progress;
  });
  handleCheckUpdate(false);
  try { activeEmail.value = await getActiveAccount(); } catch (_) {}
  try {
    const v = await getSetting("enable_groups");
    enableGroups.value = v === "true";
    if (enableGroups.value) await loadGroups();
  } catch (_) {}
})
</script>

<template>
  <div class="h-screen flex flex-col bg-gray-50 text-gray-800 select-none">
    <!-- Header -->
    <header class="flex-shrink-0 border-b border-gray-200 bg-white px-6 py-4">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-xl font-bold text-gray-900">账号管理</h1>
          <p class="text-sm text-gray-500 mt-0.5">
            共 {{ stats.total }} 个账号
            <span v-if="stats.free" class="text-green-600 ml-2">Free: {{ stats.free }}</span>
            <span v-if="stats.trial" class="text-orange-500 ml-2">Trial: {{ stats.trial }}</span>
            <span v-if="stats.pro" class="text-blue-600 ml-2">Pro: {{ stats.pro }}</span>
          </p>
        </div>
        <div class="flex items-center gap-2">
          <button
            @click="handleBatchRefresh"
            :disabled="batchRefreshing"
            :class="[
              'inline-flex items-center gap-1.5 px-3 py-1.5 text-sm border rounded-lg transition-colors',
              batchRefreshing
                ? 'border-gray-200 text-gray-400 cursor-not-allowed'
                : 'border-green-300 text-green-700 hover:bg-green-50',
            ]"
          >
            <svg class="w-4 h-4" :class="{ 'animate-spin': batchRefreshing }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <template v-if="batchRefreshing">
              {{ batchRefreshProgress.done }}/{{ batchRefreshProgress.total }}
              <span v-if="batchRefreshProgress.failed" class="text-red-500">({{ batchRefreshProgress.failed }}失败)</span>
            </template>
            <template v-else>
              刷新配额{{ selectedIds.length > 0 ? ` (${selectedIds.length})` : '' }}
            </template>
          </button>
          <button
            v-if="selectedIds.length > 0"
            @click="handleDeleteSelected"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            删除 ({{ selectedIds.length }})
          </button>
          <button
            @click="showAddDialog = true"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            添加账号
          </button>
          <button
            @click="openUrl('https://shop.xiaobiao.ltd')"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors shadow-sm"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 100 4 2 2 0 000-4z" />
            </svg>
            购买账号
          </button>
          <button
            v-if="enableGroups && selectedIds.length > 0"
            @click="showSetGroup = true"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm border border-purple-300 text-purple-700 hover:bg-purple-50 rounded-lg transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
            </svg>
            设置分组 ({{ selectedIds.length }})
          </button>
        </div>
      </div>

      <!-- Search & Filter -->
      <div class="flex items-center gap-3 mt-3">
        <div class="relative flex-1 max-w-sm">
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索邮箱..."
            class="w-full pl-9 pr-3 py-1.5 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>
        <div class="flex gap-1">
          <button
            v-for="t in [
              { key: 'all', label: '全部' },
              { key: 'Free', label: 'Free' },
              { key: 'Trial', label: 'Trial' },
              { key: 'Unknown', label: '未知' },
            ]"
            :key="t.key"
            @click="filterType = t.key"
            :class="[
              'px-3 py-1 text-sm rounded-full transition-colors',
              filterType === t.key
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-600 hover:bg-gray-200',
            ]"
          >
            {{ t.label }}
          </button>
        </div>

        <!-- Group filter -->
        <template v-if="enableGroups">
          <span class="text-gray-300">|</span>
          <select
            v-model="filterGroup"
            class="px-3 py-1 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white"
          >
            <option value="all">全部分组</option>
            <option value="__ungrouped__">未分组</option>
            <option v-for="g in groups" :key="g" :value="g">{{ g }}</option>
          </select>
          <button
            @click="showGroupManager = true"
            class="p-1 text-gray-400 hover:text-gray-600 transition-colors"
            title="管理分组"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </template>
      </div>
    </header>

    <!-- Table -->
    <main class="flex-1 overflow-auto">
      <AccountTable
        :accounts="filteredAccounts"
        :refreshingIds="refreshingIds"
        :activeEmail="activeEmail"
        :enableGroups="enableGroups"
        v-model:selectedIds="selectedIds"
        @switch="handleSwitch"
        @refresh-quota="handleRefreshSingle"
        @delete="(id) => { deleteAccounts([id]).then(loadAccounts); }"
      />
    </main>

    <!-- Footer -->
    <footer class="flex items-center justify-center gap-4 px-4 py-2 text-xs text-gray-400 border-t border-gray-100 shrink-0">
      <a
        href="#"
        @click.prevent="openUrl('https://github.com/dadadadadadaddadadadadadadadadaddada/windsurf-account-manager')"
        class="inline-flex items-center gap-1 hover:text-gray-600 transition-colors"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
        </svg>
        GitHub
      </a>
      <span class="text-gray-300">|</span>
      <span
        class="cursor-pointer hover:text-gray-600 transition-colors"
        title="点击复制群号"
        @click="navigator.clipboard.writeText('686141959')"
      >QQ群: 686141959</span>
      <span class="text-gray-300">|</span>
      <button
        @click="showSettings = true"
        class="inline-flex items-center gap-1 hover:text-gray-600 transition-colors"
        title="设置"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        设置
      </button>
      <span class="text-gray-300">|</span>
      <button
        @click="handleCheckUpdate(true)"
        class="inline-flex items-center gap-1 hover:text-gray-600 transition-colors"
        title="检查更新"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        检查更新
      </button>
    </footer>

    <!-- Dialogs -->
    <AddAccountDialog
      v-if="showAddDialog"
      @close="showAddDialog = false"
      @submit="handleAddAccount"
    />
    <SwitchProgressDialog v-if="switching" :progress="switchProgress" />
    <SettingsDialog
      v-if="showSettings"
      :enableGroups="enableGroups"
      @close="showSettings = false"
      @update:enableGroups="(v) => { enableGroups = v; if (v) loadGroups(); }"
      @groupsChanged="loadGroups(); loadAccounts();"
    />
    <UpdateDialog v-if="updateInfo" :info="updateInfo" @close="updateInfo = null" />
    <SetGroupDialog
      v-if="showSetGroup"
      :ids="selectedIds"
      @close="showSetGroup = false"
      @done="showSetGroup = false; loadAccounts(); loadGroups();"
    />
    <GroupManagerDialog
      v-if="showGroupManager"
      @close="showGroupManager = false"
      @done="loadAccounts(); loadGroups();"
    />

    <!-- 通用确认弹框 -->
    <Teleport to="body">
      <div v-if="confirmDialog" class="fixed inset-0 z-50 flex items-center justify-center">
        <div class="absolute inset-0 bg-black/40" @click="confirmDialog = null"></div>
        <div class="relative bg-white rounded-xl shadow-xl p-6 w-80 max-w-[90vw]">
          <h3 class="text-base font-semibold text-gray-900 mb-2">{{ confirmDialog.title }}</h3>
          <p class="text-sm text-gray-600 mb-4 whitespace-pre-wrap break-all max-h-60 overflow-y-auto">{{ confirmDialog.message }}</p>
          <div class="flex justify-end gap-2">
            <button
              @click="confirmDialog = null"
              class="px-3 py-1.5 text-sm text-gray-600 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
            >
              取消
            </button>
            <button
              @click="confirmDialog.onConfirm()"
              class="px-3 py-1.5 text-sm text-white bg-red-500 hover:bg-red-600 rounded-lg transition-colors"
            >
              确定
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
