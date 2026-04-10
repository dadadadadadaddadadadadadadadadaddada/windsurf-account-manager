<script setup>
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps({
  info: { type: Object, required: true },
});

const emit = defineEmits(["close"]);
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/40" @click="emit('close')"></div>
      <div class="relative bg-white rounded-xl shadow-xl w-[420px] max-w-[90vw]">
        <!-- Header -->
        <div class="flex items-center gap-2 px-5 py-4 border-b border-gray-100">
          <svg class="w-5 h-5 text-green-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
          </svg>
          <h3 class="text-base font-semibold text-gray-900">发现新版本</h3>
        </div>

        <!-- Body -->
        <div class="px-5 py-4 space-y-3">
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-500">当前版本:</span>
            <span class="text-sm font-mono text-gray-700">v{{ info.current_version }}</span>
            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
            </svg>
            <span class="text-sm font-mono text-green-600 font-semibold">v{{ info.remote_version }}</span>
          </div>

          <div v-if="info.update_content">
            <p class="text-sm font-medium text-gray-700 mb-1">更新内容:</p>
            <div class="bg-gray-50 rounded-lg px-3 py-2 text-sm text-gray-600 whitespace-pre-wrap max-h-48 overflow-y-auto leading-relaxed">{{ info.update_content }}</div>
          </div>
        </div>

        <!-- Footer -->
        <div class="flex justify-end gap-2 px-5 py-3 border-t border-gray-100">
          <button
            @click="emit('close')"
            class="px-4 py-2 text-sm text-gray-600 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
          >
            稍后再说
          </button>
          <button
            v-if="info.download_url"
            @click="openUrl(info.download_url); emit('close')"
            class="px-4 py-2 text-sm text-white bg-green-600 hover:bg-green-700 rounded-lg transition-colors"
          >
            前往下载
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
