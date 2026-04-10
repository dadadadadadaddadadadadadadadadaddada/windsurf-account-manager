<script setup>
import { ref, onMounted } from "vue";
import { listGroups, setAccountsGroup } from "../lib/tauri";

const props = defineProps({
  ids: { type: Array, required: true },
});
const emit = defineEmits(["close", "done"]);

const groups = ref([]);
const selected = ref("");
const saving = ref(false);

onMounted(async () => {
  try {
    groups.value = await listGroups();
  } catch (_) {}
});

async function handleConfirm() {
  saving.value = true;
  try {
    await setAccountsGroup(props.ids, selected.value);
    emit("done");
  } catch (e) {
    console.error("设置分组失败:", e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/40" @click="emit('close')"></div>
      <div class="relative bg-white rounded-xl shadow-xl w-[380px] max-w-[90vw]">
        <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100">
          <h3 class="text-base font-semibold text-gray-900">设置分组 ({{ ids.length }}个账号)</h3>
          <button @click="emit('close')" class="text-gray-400 hover:text-gray-600">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="px-5 py-4">
          <div class="space-y-1 max-h-48 overflow-y-auto">
            <label
              class="flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer transition-colors"
              :class="selected === '' ? 'bg-blue-50 border border-blue-200' : 'hover:bg-gray-50 border border-transparent'"
            >
              <input type="radio" v-model="selected" value="" class="w-4 h-4 text-blue-600" />
              <span class="text-sm text-gray-500 italic">未分组</span>
            </label>
            <label
              v-for="g in groups"
              :key="g"
              class="flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer transition-colors"
              :class="selected === g ? 'bg-blue-50 border border-blue-200' : 'hover:bg-gray-50 border border-transparent'"
            >
              <input type="radio" v-model="selected" :value="g" class="w-4 h-4 text-blue-600" />
              <span class="text-sm text-gray-700">{{ g }}</span>
            </label>
          </div>
        </div>

        <div class="flex justify-end gap-2 px-5 py-3 border-t border-gray-100">
          <button @click="emit('close')" class="px-4 py-2 text-sm text-gray-600 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors">取消</button>
          <button
            @click="handleConfirm"
            :disabled="saving"
            class="px-4 py-2 text-sm text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50"
          >
            {{ saving ? '保存中...' : '确认' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
