<script setup>
import { ref, computed } from "vue";

const props = defineProps({
  accounts: { type: Array, default: () => [] },
  selectedIds: { type: Array, default: () => [] },
  refreshingIds: { type: Set, default: () => new Set() },
  activeEmail: { type: String, default: null },
  enableGroups: { type: Boolean, default: false },
});

const emit = defineEmits(["update:selectedIds", "switch", "refresh-quota", "delete"]);

const copiedField = ref(null);
const pendingDeleteAccount = ref(null);

const allSelected = computed(() => {
  return (
    props.accounts.length > 0 &&
    props.accounts.every((a) => props.selectedIds.includes(a.id))
  );
});

function toggleAll() {
  if (allSelected.value) {
    emit("update:selectedIds", []);
  } else {
    emit("update:selectedIds", props.accounts.map((a) => a.id));
  }
}

function confirmDelete(account) {
  pendingDeleteAccount.value = account;
}

function doDelete() {
  if (pendingDeleteAccount.value) {
    emit('delete', pendingDeleteAccount.value.id);
    pendingDeleteAccount.value = null;
  }
}

function toggleOne(id) {
  const current = [...props.selectedIds];
  const idx = current.indexOf(id);
  if (idx >= 0) {
    current.splice(idx, 1);
  } else {
    current.push(id);
  }
  emit("update:selectedIds", current);
}

function isSelected(id) {
  return props.selectedIds.includes(id);
}

function getTypeBadgeClass(type) {
  switch (type) {
    case "Free":
      return "bg-green-50 text-green-700 border-green-200";
    case "Trial":
      return "bg-orange-50 text-orange-700 border-orange-200";
    case "Pro":
      return "bg-blue-50 text-blue-700 border-blue-200";
    default:
      return "bg-gray-50 text-gray-600 border-gray-200";
  }
}

function copyText(text, fieldKey) {
  navigator.clipboard.writeText(text);
  copiedField.value = fieldKey;
  setTimeout(() => {
    copiedField.value = null;
  }, 1500);
}

function formatResetTime(timestamp) {
  if (!timestamp || timestamp <= 0) return "-";
  const d = new Date(timestamp * 1000);
  const month = d.getMonth() + 1;
  const day = d.getDate();
  const hours = String(d.getHours()).padStart(2, "0");
  const mins = String(d.getMinutes()).padStart(2, "0");
  return `${month}/${day} ${hours}:${mins}`;
}

function quotaColor(pct) {
  if (pct < 0) return "bg-gray-200";
  if (pct <= 20) return "bg-red-500";
  if (pct <= 50) return "bg-orange-400";
  return "bg-green-500";
}

function quotaTextColor(pct) {
  if (pct < 0) return "text-gray-400";
  if (pct <= 20) return "text-red-600";
  if (pct <= 50) return "text-orange-600";
  return "text-green-600";
}

function formatExpiresAt(timestamp) {
  if (!timestamp || timestamp <= 0) return "-";
  const d = new Date(timestamp * 1000);
  const now = new Date();
  const diffDays = Math.ceil((d - now) / (1000 * 60 * 60 * 24));
  const dateStr = `${d.getFullYear()}/${d.getMonth() + 1}/${d.getDate()}`;
  if (diffDays < 0) return dateStr;
  if (diffDays <= 3) return dateStr;
  return dateStr;
}

function expiresColor(timestamp) {
  if (!timestamp || timestamp <= 0) return "text-gray-400";
  const now = Date.now() / 1000;
  const days = (timestamp - now) / 86400;
  if (days < 0) return "text-red-500";
  if (days <= 3) return "text-orange-500";
  if (days <= 7) return "text-yellow-600";
  return "text-gray-600";
}

function expiresLabel(timestamp) {
  if (!timestamp || timestamp <= 0) return "";
  const now = Date.now() / 1000;
  const days = Math.ceil((timestamp - now) / 86400);
  if (days < 0) return `已过期${-days}天`;
  if (days === 0) return "今天到期";
  if (days <= 30) return `剩${days}天`;
  return "";
}
</script>

<template>
  <div class="min-w-full">
    <table class="w-full text-sm">
      <thead class="bg-gray-50 sticky top-0 z-10">
        <tr class="border-b border-gray-200">
          <th class="w-10 px-4 py-3 text-left">
            <input
              type="checkbox"
              :checked="allSelected"
              @change="toggleAll"
              class="w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 cursor-pointer"
            />
          </th>
          <th class="px-4 py-3 text-left font-medium text-gray-600 w-64">邮箱</th>
          <th class="px-4 py-3 text-left font-medium text-gray-600 w-24">密码</th>
          <th class="px-4 py-3 text-left font-medium text-gray-600 w-20">类型</th>
          <th class="px-4 py-3 text-left font-medium text-gray-600 w-32">日额度</th>
          <th class="px-4 py-3 text-left font-medium text-gray-600 w-32">周额度</th>
          <th class="px-4 py-3 text-left font-medium text-gray-600 w-28">到期日</th>
          <th class="px-4 py-3 text-right font-medium text-gray-600 w-40">操作</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="account in accounts"
          :key="account.id"
          class="border-b border-gray-100 transition-colors"
          :class="[
            account.email === activeEmail
              ? 'bg-green-100 hover:bg-green-200/80'
              : isSelected(account.id)
                ? 'bg-blue-50/50 hover:bg-blue-50/30'
                : 'hover:bg-blue-50/30'
          ]"
        >
          <td class="px-4 py-3">
            <input
              type="checkbox"
              :checked="isSelected(account.id)"
              @change="toggleOne(account.id)"
              class="w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 cursor-pointer"
            />
          </td>
          <td class="px-4 py-3">
            <div class="flex items-center gap-2">
              <span
                @click="copyText(account.email, 'email-' + account.id)"
                class="font-mono text-gray-800 cursor-pointer hover:text-blue-600 transition-colors"
                title="点击复制"
              >
                {{ account.email }}
                <span v-if="copiedField === 'email-' + account.id" class="ml-1 text-xs text-green-500">已复制</span>
              </span>
              <span
                v-if="account.email === activeEmail"
                class="inline-flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] font-medium text-green-700 bg-green-100 border border-green-200 rounded-full shrink-0"
              >
                <svg class="w-2.5 h-2.5" fill="currentColor" viewBox="0 0 8 8"><circle cx="4" cy="4" r="3"/></svg>
                使用中
              </span>
              <span
                v-if="enableGroups && account.group_name"
                class="inline-flex items-center px-1.5 py-0.5 text-[10px] font-medium text-purple-700 bg-purple-50 border border-purple-200 rounded-full shrink-0"
              >
                {{ account.group_name }}
              </span>
            </div>
          </td>
          <td class="px-4 py-3">
            <span
              @click="copyText(account.password, 'pwd-' + account.id)"
              class="font-mono text-gray-400 cursor-pointer hover:text-blue-600 transition-colors"
              title="点击复制密码"
            >
              ••••••
              <span v-if="copiedField === 'pwd-' + account.id" class="ml-1 text-xs text-green-500">已复制</span>
            </span>
          </td>
          <td class="px-4 py-3">
            <span
              :class="[
                'inline-block px-2 py-0.5 text-xs font-medium rounded-full border',
                getTypeBadgeClass(account.account_type),
              ]"
            >
              {{ account.account_type || "Unknown" }}
            </span>
          </td>
          <td class="px-4 py-3">
            <div v-if="account.daily_remaining >= 0" class="flex flex-col gap-0.5">
              <div class="flex items-center gap-1.5">
                <div class="flex-1 h-1.5 bg-gray-100 rounded-full overflow-hidden">
                  <div :class="['h-full rounded-full', quotaColor(account.daily_remaining)]" :style="{ width: account.daily_remaining + '%' }"></div>
                </div>
                <span :class="['text-xs font-medium w-8 text-right', quotaTextColor(account.daily_remaining)]">{{ account.daily_remaining }}%</span>
              </div>
              <span class="text-[10px] text-gray-400">{{ formatResetTime(account.daily_reset_at) }}</span>
            </div>
            <span v-else class="text-gray-400">-</span>
          </td>
          <td class="px-4 py-3">
            <div v-if="account.weekly_remaining >= 0" class="flex flex-col gap-0.5">
              <div class="flex items-center gap-1.5">
                <div class="flex-1 h-1.5 bg-gray-100 rounded-full overflow-hidden">
                  <div :class="['h-full rounded-full', quotaColor(account.weekly_remaining)]" :style="{ width: account.weekly_remaining + '%' }"></div>
                </div>
                <span :class="['text-xs font-medium w-8 text-right', quotaTextColor(account.weekly_remaining)]">{{ account.weekly_remaining }}%</span>
              </div>
              <span class="text-[10px] text-gray-400">{{ formatResetTime(account.weekly_reset_at) }}</span>
            </div>
            <span v-else class="text-gray-400">-</span>
          </td>
          <td class="px-4 py-3">
            <div v-if="account.expires_at > 0" class="flex flex-col">
              <span :class="['text-xs font-medium', expiresColor(account.expires_at)]">
                {{ formatExpiresAt(account.expires_at) }}
              </span>
              <span v-if="expiresLabel(account.expires_at)" :class="['text-[10px]', expiresColor(account.expires_at)]">
                {{ expiresLabel(account.expires_at) }}
              </span>
            </div>
            <span v-else class="text-gray-400">-</span>
          </td>
          <td class="px-4 py-3 text-right">
            <div class="flex items-center justify-end gap-1">
              <button
                @click="emit('refresh-quota', account.id)"
                :disabled="refreshingIds.has(account.id)"
                :class="[
                  'px-2 py-1 text-xs rounded transition-colors',
                  refreshingIds.has(account.id)
                    ? 'text-gray-400 cursor-not-allowed'
                    : 'text-green-600 hover:bg-green-50',
                ]"
                title="刷新配额"
              >
                <span v-if="refreshingIds.has(account.id)" class="inline-flex items-center gap-0.5">
                  <svg class="w-3 h-3 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
                  刷新中
                </span>
                <span v-else>刷新</span>
              </button>
              <button
                @click="emit('switch', account.id)"
                class="px-2 py-1 text-xs text-blue-600 hover:bg-blue-50 rounded transition-colors"
                title="切换到此账号"
              >
                切换
              </button>
              <button
                @click="confirmDelete(account)"
                class="px-2 py-1 text-xs text-red-500 hover:bg-red-50 rounded transition-colors"
                title="删除"
              >
                删除
              </button>
            </div>
          </td>
        </tr>
        <tr v-if="accounts.length === 0">
          <td colspan="8" class="px-4 py-16 text-center text-gray-400">
            <div class="flex flex-col items-center gap-2">
              <svg class="w-12 h-12 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
              </svg>
              <p>暂无账号数据</p>
              <p class="text-xs">点击右上角「添加账号」开始使用</p>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>

  <!-- 删除确认弹框 -->
  <Teleport to="body">
    <div v-if="pendingDeleteAccount" class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/40" @click="pendingDeleteAccount = null"></div>
      <div class="relative bg-white rounded-xl shadow-xl p-6 w-80 max-w-[90vw]">
        <h3 class="text-base font-semibold text-gray-900 mb-2">确认删除</h3>
        <p class="text-sm text-gray-600 mb-4">
          确定要删除账号 <span class="font-mono text-gray-800">{{ pendingDeleteAccount.email }}</span> 吗？此操作不可撤销。
        </p>
        <div class="flex justify-end gap-2">
          <button
            @click="pendingDeleteAccount = null"
            class="px-3 py-1.5 text-sm text-gray-600 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
          >
            取消
          </button>
          <button
            @click="doDelete"
            class="px-3 py-1.5 text-sm text-white bg-red-500 hover:bg-red-600 rounded-lg transition-colors"
          >
            删除
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
