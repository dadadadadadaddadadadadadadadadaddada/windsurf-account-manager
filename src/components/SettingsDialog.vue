<script setup>
import { ref, onMounted } from "vue";
import { getWindsurfPath, setWindsurfPath } from "../lib/tauri";

const emit = defineEmits(["close"]);

const customPath = ref("");
const detectedPath = ref("");
const effectivePath = ref("");
const saving = ref(false);
const saved = ref(false);

onMounted(async () => {
  try {
    const info = await getWindsurfPath();
    customPath.value = info.custom || "";
    detectedPath.value = info.detected || "";
    effectivePath.value = info.effective || "";
  } catch (e) {
    console.error("Failed to get windsurf path:", e);
  }
});

async function handleSave() {
  saving.value = true;
  saved.value = false;
  try {
    await setWindsurfPath(customPath.value.trim());
    saved.value = true;
    setTimeout(() => { saved.value = false; }, 2000);
    const info = await getWindsurfPath();
    effectivePath.value = info.effective || "";
  } catch (e) {
    console.error("Failed to save path:", e);
  } finally {
    saving.value = false;
  }
}

async function handleBrowse() {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Windsurf", extensions: ["exe", "app", ""] }
      ],
    });
    if (selected) {
      customPath.value = selected;
    }
  } catch (e) {
    console.error("File dialog error:", e);
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/40" @click="emit('close')"></div>
      <div class="relative bg-white rounded-xl shadow-xl w-[480px] max-w-[90vw]">
        <!-- Header -->
        <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100">
          <h3 class="text-base font-semibold text-gray-900">设置</h3>
          <button @click="emit('close')" class="text-gray-400 hover:text-gray-600 transition-colors">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Body -->
        <div class="px-5 py-4 space-y-4">
          <!-- Auto detect status -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">自动检测路径</label>
            <div class="flex items-center gap-2">
              <span v-if="detectedPath" class="text-sm text-green-600 break-all">{{ detectedPath }}</span>
              <span v-else class="text-sm text-red-500">未检测到 Windsurf 安装路径</span>
            </div>
          </div>

          <!-- Custom path input -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">手动配置路径</label>
            <p class="text-xs text-gray-400 mb-2">
              自动检测失败时，请手动指定 Windsurf 可执行文件路径
            </p>
            <div class="flex gap-2">
              <input
                v-model="customPath"
                type="text"
                placeholder="例如: C:\Program Files\Windsurf\Windsurf.exe"
                class="flex-1 px-3 py-2 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
              <button
                @click="handleBrowse"
                class="px-3 py-2 text-sm bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors whitespace-nowrap"
              >
                浏览
              </button>
            </div>
          </div>

          <!-- Effective path -->
          <div v-if="effectivePath" class="bg-gray-50 rounded-lg px-3 py-2">
            <span class="text-xs text-gray-500">当前生效路径: </span>
            <span class="text-xs text-gray-700 break-all">{{ effectivePath }}</span>
          </div>
        </div>

        <!-- Footer -->
        <div class="flex justify-end gap-2 px-5 py-3 border-t border-gray-100">
          <button
            @click="emit('close')"
            class="px-4 py-2 text-sm text-gray-600 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
          >
            取消
          </button>
          <button
            @click="handleSave"
            :disabled="saving"
            class="px-4 py-2 text-sm text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50"
          >
            {{ saved ? '已保存 ✓' : saving ? '保存中...' : '保存' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
