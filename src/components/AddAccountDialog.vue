<script setup>
import { ref, computed } from "vue";

const emit = defineEmits(["close", "submit"]);

const pasteText = ref("");

const parsedAccounts = computed(() => {
  const lines = pasteText.value
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l);
  return lines.map((line) => {
    const parts = line.split("----");
    const email = parts[0].trim();
    const password = parts.length > 1 ? parts[1].trim() : email;
    return { email, password };
  });
});

function handleSubmit() {
  if (parsedAccounts.value.length === 0) {
    alert("请粘贴至少一行账号数据");
    return;
  }
  for (const acc of parsedAccounts.value) {
    emit("submit", {
      email: acc.email,
      password: acc.password,
      refresh_token: "",
      account_type: "Unknown",
    });
  }
}
</script>

<template>
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <div
      class="bg-white rounded-xl shadow-2xl w-full max-w-lg mx-4 overflow-hidden"
    >
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-100">
        <h2 class="text-lg font-semibold text-gray-900">添加账号</h2>
        <button
          @click="emit('close')"
          class="p-1 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-gray-100 transition-colors"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="px-6 py-5 space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            粘贴账号数据 <span class="text-gray-400 font-normal">（每行一个）</span>
          </label>
          <textarea
            v-model="pasteText"
            rows="10"
            :placeholder="'账号----密码\n账号（密码与账号相同）\n\n示例:\nzft001@yahoo.com----o7VtnXaEkb\nzft002@yahoo.com'"
            class="w-full px-3 py-2 text-sm font-mono border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
          />
        </div>

        <div class="rounded-lg bg-gray-50 p-3 text-xs text-gray-500 space-y-1">
          <p class="font-medium text-gray-600">支持格式:</p>
          <p><span class="font-mono text-gray-700">邮箱----密码</span> — 用 ---- 分隔账号和密码</p>
          <p><span class="font-mono text-gray-700">邮箱</span> — 密码默认与账号相同</p>
        </div>

        <div v-if="parsedAccounts.length > 0" class="text-sm text-gray-600">
          已识别 <span class="font-semibold text-blue-600">{{ parsedAccounts.length }}</span> 个账号
        </div>
      </div>

      <div class="flex justify-end gap-2 px-6 py-4 border-t border-gray-100 bg-gray-50">
        <button
          @click="emit('close')"
          class="px-4 py-2 text-sm text-gray-600 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
        >
          取消
        </button>
        <button
          @click="handleSubmit"
          :disabled="parsedAccounts.length === 0"
          :class="[
            'px-4 py-2 text-sm text-white rounded-lg transition-colors',
            parsedAccounts.length > 0
              ? 'bg-blue-600 hover:bg-blue-700'
              : 'bg-gray-300 cursor-not-allowed',
          ]"
        >
          添加 {{ parsedAccounts.length > 0 ? `(${parsedAccounts.length})` : '' }}
        </button>
      </div>
    </div>
  </div>
</template>
