<script setup>
defineProps({
  progress: {
    type: Object,
    default: () => ({ step: 0, total: 5, message: "" }),
  },
});

function getStepLabel(step) {
  const labels = {
    1: "关闭 Windsurf",
    2: "重置机器 ID",
    3: "获取凭证",
    4: "写入认证数据",
    5: "启动 Windsurf",
  };
  return labels[step] || "";
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
    <div class="bg-white rounded-xl shadow-2xl w-full max-w-sm mx-4 overflow-hidden">
      <div class="px-6 py-5">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">正在切换账号</h3>

        <div class="space-y-3">
          <div v-for="i in progress.total" :key="i" class="flex items-center gap-3">
            <div
              :class="[
                'w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium flex-shrink-0',
                i < progress.step
                  ? 'bg-green-500 text-white'
                  : i === progress.step
                    ? 'bg-blue-500 text-white animate-pulse'
                    : 'bg-gray-200 text-gray-400',
              ]"
            >
              <svg
                v-if="i < progress.step"
                class="w-3.5 h-3.5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
              </svg>
              <span v-else>{{ i }}</span>
            </div>
            <span
              :class="[
                'text-sm',
                i <= progress.step ? 'text-gray-800' : 'text-gray-400',
              ]"
            >
              {{ getStepLabel(i) }}
            </span>
          </div>
        </div>

        <div class="mt-4 pt-3 border-t border-gray-100">
          <p class="text-sm text-gray-500">{{ progress.message }}</p>
          <div class="mt-2 w-full bg-gray-200 rounded-full h-1.5">
            <div
              class="bg-blue-500 h-1.5 rounded-full transition-all duration-500"
              :style="{ width: `${(progress.step / progress.total) * 100}%` }"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
