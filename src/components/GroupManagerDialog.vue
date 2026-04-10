<script setup>
import { ref, onMounted } from "vue";
import { listGroups, renameGroup, deleteGroup } from "../lib/tauri";

const emit = defineEmits(["close", "done"]);

const groups = ref([]);
const editingGroup = ref(null);
const editName = ref("");
const loading = ref(false);

async function load() {
  try {
    groups.value = await listGroups();
  } catch (_) {}
}

onMounted(load);

function startEdit(name) {
  editingGroup.value = name;
  editName.value = name;
}

function cancelEdit() {
  editingGroup.value = null;
  editName.value = "";
}

async function handleRename(oldName) {
  const newName = editName.value.trim();
  if (!newName || newName === oldName) { cancelEdit(); return; }
  loading.value = true;
  try {
    await renameGroup(oldName, newName);
    await load();
    cancelEdit();
    emit("done");
  } catch (e) {
    console.error("重命名分组失败:", e);
  } finally {
    loading.value = false;
  }
}

async function handleDelete(name) {
  loading.value = true;
  try {
    await deleteGroup(name);
    await load();
    emit("done");
  } catch (e) {
    console.error("删除分组失败:", e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center">
      <div class="absolute inset-0 bg-black/40" @click="emit('close')"></div>
      <div class="relative bg-white rounded-xl shadow-xl w-[380px] max-w-[90vw]">
        <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100">
          <h3 class="text-base font-semibold text-gray-900">管理分组</h3>
          <button @click="emit('close')" class="text-gray-400 hover:text-gray-600">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="px-5 py-4">
          <div v-if="groups.length === 0" class="text-sm text-gray-400 text-center py-6">暂无分组</div>
          <div v-else class="space-y-1 max-h-64 overflow-y-auto">
            <div
              v-for="g in groups"
              :key="g"
              class="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-gray-50 group"
            >
              <template v-if="editingGroup === g">
                <input
                  v-model="editName"
                  type="text"
                  class="flex-1 px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                  @keyup.enter="handleRename(g)"
                  @keyup.escape="cancelEdit"
                  autofocus
                />
                <button @click="handleRename(g)" class="text-green-600 hover:text-green-700" :disabled="loading">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                </button>
                <button @click="cancelEdit" class="text-gray-400 hover:text-gray-600">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </template>
              <template v-else>
                <span class="flex-1 text-sm text-gray-700">{{ g }}</span>
                <button
                  @click="startEdit(g)"
                  class="text-gray-400 hover:text-blue-600 opacity-0 group-hover:opacity-100 transition-opacity"
                  title="重命名"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button
                  @click="handleDelete(g)"
                  class="text-gray-400 hover:text-red-600 opacity-0 group-hover:opacity-100 transition-opacity"
                  title="删除分组（账号不会被删除）"
                  :disabled="loading"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </template>
            </div>
          </div>
        </div>

        <div class="flex justify-end px-5 py-3 border-t border-gray-100">
          <button @click="emit('close')" class="px-4 py-2 text-sm text-gray-600 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors">关闭</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
