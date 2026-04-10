<script setup>
import { ref, onMounted } from "vue";
import { getWindsurfPath, setWindsurfPath, getSetting, setSetting, listGroups, renameGroup, deleteGroup, createGroup } from "../lib/tauri";

const props = defineProps({
  enableGroups: { type: Boolean, default: false },
});
const emit = defineEmits(["close", "update:enableGroups", "groupsChanged"]);

const customPath = ref("");
const detectedPath = ref("");
const effectivePath = ref("");
const saving = ref(false);
const saved = ref(false);
const groupsEnabled = ref(false);
const groups = ref([]);
const newGroupName = ref("");
const editingGroup = ref(null);
const editName = ref("");
const groupLoading = ref(false);

async function loadGroupList() {
  try { groups.value = await listGroups(); } catch (_) {}
}

async function handleAddGroup() {
  const name = newGroupName.value.trim();
  if (!name) return;
  if (groups.value.includes(name)) return;
  groupLoading.value = true;
  try {
    await createGroup(name);
    newGroupName.value = "";
    await loadGroupList();
    emit("groupsChanged");
  } catch (e) {
    console.error("创建分组失败:", e);
  } finally {
    groupLoading.value = false;
  }
}

function startEdit(name) {
  editingGroup.value = name;
  editName.value = name;
}

function cancelEdit() {
  editingGroup.value = null;
  editName.value = "";
}

async function handleRename(oldName) {
  const nn = editName.value.trim();
  if (!nn || nn === oldName) { cancelEdit(); return; }
  groupLoading.value = true;
  try {
    await renameGroup(oldName, nn);
    await loadGroupList();
    cancelEdit();
    emit("groupsChanged");
  } catch (e) {
    console.error("重命名失败:", e);
  } finally {
    groupLoading.value = false;
  }
}

async function handleDeleteGroup(name) {
  groupLoading.value = true;
  try {
    await deleteGroup(name);
    await loadGroupList();
    emit("groupsChanged");
  } catch (e) {
    console.error("删除分组失败:", e);
  } finally {
    groupLoading.value = false;
  }
}

onMounted(async () => {
  try {
    const info = await getWindsurfPath();
    customPath.value = info.custom || "";
    detectedPath.value = info.detected || "";
    effectivePath.value = info.effective || "";
  } catch (e) {
    console.error("Failed to get windsurf path:", e);
  }
  groupsEnabled.value = props.enableGroups;
  if (props.enableGroups) await loadGroupList();
});

async function handleSave() {
  saving.value = true;
  saved.value = false;
  try {
    await setWindsurfPath(customPath.value.trim());
    await setSetting("enable_groups", groupsEnabled.value ? "true" : "false");
    saved.value = true;
    setTimeout(() => { saved.value = false; }, 2000);
    const info = await getWindsurfPath();
    effectivePath.value = info.effective || "";
    emit("update:enableGroups", groupsEnabled.value);
  } catch (e) {
    console.error("Failed to save:", e);
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

          <!-- Divider -->
          <div class="border-t border-gray-100"></div>

          <!-- Enable groups toggle -->
          <div class="flex items-center justify-between">
            <div>
              <label class="block text-sm font-medium text-gray-700">启用账号分组</label>
              <p class="text-xs text-gray-400 mt-0.5">开启后可对账号进行自定义分组管理</p>
            </div>
            <button
              @click="groupsEnabled = !groupsEnabled; if (groupsEnabled) loadGroupList();"
              :class="[
                'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                groupsEnabled ? 'bg-blue-600' : 'bg-gray-300',
              ]"
            >
              <span
                :class="[
                  'inline-block h-4 w-4 transform rounded-full bg-white transition-transform shadow',
                  groupsEnabled ? 'translate-x-6' : 'translate-x-1',
                ]"
              />
            </button>
          </div>

          <!-- Group management (visible when enabled) -->
          <template v-if="groupsEnabled">
            <div class="border border-gray-200 rounded-lg">
              <div class="px-3 py-2 bg-gray-50 border-b border-gray-200 rounded-t-lg">
                <span class="text-sm font-medium text-gray-700">分组管理</span>
              </div>

              <!-- Add new group -->
              <div class="flex gap-2 px-3 py-2 border-b border-gray-100">
                <input
                  v-model="newGroupName"
                  type="text"
                  placeholder="输入新分组名称"
                  class="flex-1 px-2 py-1.5 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  @keyup.enter="handleAddGroup"
                />
                <button
                  @click="handleAddGroup"
                  :disabled="groupLoading || !newGroupName.trim()"
                  class="px-3 py-1.5 text-sm text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50 whitespace-nowrap"
                >
                  添加
                </button>
              </div>

              <!-- Group list -->
              <div class="max-h-40 overflow-y-auto">
                <div v-if="groups.length === 0" class="text-sm text-gray-400 text-center py-4">暂无分组</div>
                <div
                  v-for="g in groups"
                  :key="g"
                  class="flex items-center gap-2 px-3 py-2 border-b border-gray-50 last:border-0 group hover:bg-gray-50"
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
                    <button @click="handleRename(g)" class="text-green-600 hover:text-green-700 p-1" :disabled="groupLoading">
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      </svg>
                    </button>
                    <button @click="cancelEdit" class="text-gray-400 hover:text-gray-600 p-1">
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </template>
                  <template v-else>
                    <span class="flex-1 text-sm text-gray-700">{{ g }}</span>
                    <button
                      @click="startEdit(g)"
                      class="text-gray-400 hover:text-blue-600 p-1 opacity-0 group-hover:opacity-100 transition-opacity"
                      title="重命名"
                    >
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      @click="handleDeleteGroup(g)"
                      class="text-gray-400 hover:text-red-600 p-1 opacity-0 group-hover:opacity-100 transition-opacity"
                      title="删除分组（账号不会被删除）"
                      :disabled="groupLoading"
                    >
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </template>
                </div>
              </div>
            </div>
          </template>
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
