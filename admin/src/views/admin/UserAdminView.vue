<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from "element-plus";
import { formatTime } from "@/utils";
import { getUsers, createUser, updateUser, deleteUser ,getClasses} from "@/apis";
import { getRoles } from "@/apis/roles";
import { useI18n } from "vue-i18n";
import type { User, Role, UserRoleInfo ,ClassInfo, UserClassInfo} from "@/types";
const rows = ref<User[]>([]);
const selectedIds = ref<number[]>([]);
const roles = ref<Role[]>([]);
const classes = ref<ClassInfo[]>([]);
const page = ref(1);
const pageSize = ref(20);
const total = ref(0);
const { t } = useI18n();
const query = reactive({ username: "" as string | undefined });
async function reload() {
  const data = await getUsers({ page: page.value, page_size: pageSize.value, username: query.username || undefined });
  rows.value = data.list;
  total.value = data.total;
}
async function reloadRoles() {
  const data = await getRoles({ page: 1, page_size: 1000 });
  roles.value = data.list;
}
async function reloadClasses() {
  const data = await getClasses({ page: 1, page_size: 1000 });
  classes.value = data.list;
}
function resetFilters() {
  query.username = "";
  page.value = 1;
  reload();
}
function onSelChange(arr: User[]) {
  selectedIds.value = arr.map((it) => it.id);
}
function handlePageChange(p: number) {
  page.value = p;
  reload();
}
function handleSizeChange(s: number) {
  pageSize.value = s;
  page.value = 1;
  reload();
}
const dialog = reactive({ visible: false, mode: "create" as "create" | "edit", editingId: undefined as number | undefined });
const formRef = ref<FormInstance>();
const form = reactive<{ username: string; password?: string; role_ids: number[], class_ids: number[] }>({ username: "", password: "", role_ids: [], class_ids: [] });
const rules = reactive<FormRules>({ username: [{ required: true, message: "Username required" }] });
function openCreate() {
  dialog.mode = "create";
  dialog.editingId = undefined;
  form.username = "";
  form.password = "";
  form.role_ids = [];
  form.class_ids = [];
  dialog.visible = true;
}
function openEdit(row: User) {
  dialog.mode = "edit";
  dialog.editingId = row.id;
  form.username = row.username;
  form.password = "";
  form.role_ids = row.role_infos.map((it) => it.role_id);
  form.class_ids = row.class_infos.map((it) => it.class_id);
  dialog.visible = true;
}
async function submit() {
  const valid = await formRef.value?.validate();
  if (!valid) {
    ElMessage.error(t("common.please_check_form") as string);
    return;
  }
  if (dialog.mode === "create") {
    await createUser({ username: form.username, password: form.password || "", role_ids: form.role_ids, class_ids: form.class_ids });
    ElMessage.success(t("common.created") as string);
  } else if (dialog.editingId != null) {
    await updateUser(dialog.editingId, { username: form.username, role_ids: form.role_ids, class_ids: form.class_ids });
    ElMessage.success(t("common.save") as string);
  }
  dialog.visible = false;
  await reload();
}
async function del(id: number) {
  await ElMessageBox.confirm(t("common.delete_confirm", { name: rows.value.find((it) => it.id === id)?.username || "" }), t("common.confirm"), {
    type: "warning",
  });
  await deleteUser(id);
  ElMessage.success(t("common.deleted") as string);
  reload();
}
onMounted(() => {
  reload();
  reloadRoles();
  reloadClasses();
});
</script>

<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center gap-2">
        <el-input v-model="query.username" :placeholder="$t('common.search_by_name')" clearable class="w-64" />
        <el-button type="primary" @click="reload">{{ $t("common.search") }}</el-button>
        <el-button @click="resetFilters">{{ $t("common.reset") }}</el-button>
        <el-button type="success" @click="openCreate">{{ $t("common.new") }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="rows" stripe size="large" style="width: 100%" @selection-change="onSelChange">
        <el-table-column type="selection" width="50" />
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="username" :label="$t('user.username')" width="180" />
        <el-table-column prop="school_name" :label="$t('user.school_name')" width="180" />
        <el-table-column :label="$t('user.wechat_info')" width="220">
          <template #default="{ row }">
            <div v-if="row.wechat_nickname" class="wechat-info">
              <el-image
                style="width: 40px; height: 40px; border-radius: 50%"
                :src="row.wechat_avatar_url"
                fit="cover"
                :preview-src-list="[row.wechat_avatar_url]"
                preview-teleported
              />
              <span class="wechat-nickname">{{ row.wechat_nickname }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="role_infos" :label="$t('user.roles')">
          <template #default="{ row }">
            <el-tag v-for="role in row.role_infos" :key="role.role_id" class="role-tag">
              {{ role.role_name }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="class_infos" :label="$t('menu.classes')">
          <template #default="{ row }">
            <el-tag v-for="cls in row.class_infos" :key="cls.class_id" class="class-tag">
              {{ cls.class_name }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" :label="$t('user.created_at')" width="180">
          <template #default="{ row }">
            {{ new Date(row.created_at).toLocaleString() }}
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.operations')" width="150">
          <template #default="{ row }">
            <el-button type="primary" size="small" @click="openEdit(row)">
              {{ $t('common.edit') }}
            </el-button>
            <el-button size="small" type="danger" @click="del(row.id)">{{ $t("common.delete") }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div class="flex justify-end mt-4">
        <el-pagination
          background
          layout="total, sizes, prev, pager, next, jumper"
          :page-sizes="[10, 20, 50, 100]"
          :page-size="pageSize"
          :current-page="page"
          :total="total"
          @current-change="handlePageChange"
          @size-change="handleSizeChange"
        />
      </div>
    </el-card>

    <el-dialog v-model="dialog.visible" :title="dialog.mode === 'create' ? $t('common.create') : $t('common.edit')" width="520px">
      <el-form label-width="140px" ref="formRef" :model="form" :rules="rules">
        <el-form-item :label="$t('auth.username')" prop="username"><el-input v-model="form.username" /></el-form-item>
        <el-form-item :label="$t('menu.roles')" prop="role_ids">
          <el-select v-model="form.role_ids" multiple class="w-full">
            <el-option v-for="r in roles" :key="r.id" :label="r.name" :value="r.id" />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('menu.classes')" prop="class_ids">
          <el-select v-model="form.class_ids" multiple class="w-full">
            <el-option v-for="c in classes" :key="c.id" :label="`${c.school_name} ${c.grade}年级${c.class}班`" :value="c.id" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="dialog.mode === 'create'" :label="$t('auth.password')" prop="password"
          ><el-input v-model="form.password" type="password"
        /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog.visible = false">{{ $t("common.cancel") }}</el-button>
        <el-button type="primary" @click="submit">{{ $t("common.confirm") }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.role-tag,
.class-tag {
	margin-right: 5px;
	margin-bottom: 5px;
}

.wechat-info {
	display: flex;
	align-items: center;
}

.wechat-nickname {
	margin-left: 10px;
}
</style>
