<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from "element-plus";
import { getPermissions, createPermission, updatePermission, deletePermission } from "@/apis/permissions";
import { useI18n } from "vue-i18n";
import type { Permission, PermissionCreateRequest, PermissionUpdateRequest } from "@/types/permissions";
import { ActionType } from "@/types/permissions";

const rows = ref<Permission[]>([]);
const selectedIds = ref<number[]>([]);
const page = ref(1);
const pageSize = ref(20);
const total = ref(0);
const { t } = useI18n();

const query = reactive({ name: "" as string | undefined, resource: "" as string | undefined, action: "" as string | undefined });

async function reload() {
  const data = await getPermissions({ 
    page: page.value, 
    page_size: pageSize.value, 
    name: query.name || undefined,
    resource: query.resource || undefined,
    action: query.action || undefined,
  });
  rows.value = data.list;
  total.value = data.total;
}

function resetFilters() {
  query.name = "";
  query.resource = "";
  query.action = "";
  page.value = 1;
  reload();
}
function onSelChange(arr: Permission[]) {
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
const form = reactive<PermissionCreateRequest | PermissionUpdateRequest>({ name: "", resource: "", action: "", description: "" });
const rules = reactive<FormRules>({ 
  name: [{ required: true, message: "Name required" }],
  resource: [{ required: true, message: "Resource required" }],
  action: [{ required: true, message: "Action required" }],
});

function openCreate() {
  dialog.mode = "create";
  dialog.editingId = undefined;
  form.name = "";
  form.resource = "";
  form.action = "";
  form.description = "";
  dialog.visible = true;
}

function openEdit(row: Permission) {
  dialog.mode = "edit";
  dialog.editingId = row.id;
  form.name = row.name;
  form.resource = row.resource;
  form.action = row.action;
  form.description = row.description;
  dialog.visible = true;
}

async function submit() {
  const valid = await formRef.value?.validate();
  if (!valid) {
    ElMessage.error(t("common.please_check_form") as string);
    return;
  }
  if (dialog.mode === "create") {
    await createPermission(form as PermissionCreateRequest);
    ElMessage.success(t("common.created") as string);
  } else if (dialog.editingId != null) {
    await updatePermission(dialog.editingId, form as PermissionUpdateRequest);
    ElMessage.success(t("common.save") as string);
  }
  dialog.visible = false;
  await reload();
}

async function del(id: number) {
  await ElMessageBox.confirm(t("common.delete_confirm", { name: rows.value.find((it) => it.id === id)?.name || "" }), t("common.confirm"), {
    type: "warning",
  });
  await deletePermission(id);
  ElMessage.success(t("common.deleted") as string);
  reload();
}

onMounted(reload);
</script>

<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center gap-2">
        <el-input v-model="query.name" :placeholder="$t('common.search_by_name')" clearable class="w-48" />
        <el-input v-model="query.resource" :placeholder="$t('permissions.resource')" clearable class="w-48" />
        <el-input v-model="query.action" :placeholder="$t('permissions.action')" clearable class="w-48" />
        <el-button type="primary" @click="reload">{{ $t("common.search") }}</el-button>
        <el-button @click="resetFilters">{{ $t("common.reset") }}</el-button>
        <el-button type="success" @click="openCreate">{{ $t("common.new") }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="rows" stripe size="large" style="width: 100%" @selection-change="onSelChange">
        <el-table-column type="selection" width="50" />
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('common.name')" prop="name" min-width="160" />
        <el-table-column :label="$t('permissions.resource')" prop="resource" min-width="200" />
        <el-table-column :label="$t('permissions.action')" prop="action" min-width="120" />
        <el-table-column :label="$t('common.description')" prop="description" min-width="200" />
        <el-table-column :label="$t('common.actions')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="openEdit(row)">{{ $t("common.edit") }}</el-button>
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
        <el-form-item :label="$t('common.name')" prop="name"><el-input v-model="form.name" /></el-form-item>
        <el-form-item :label="$t('permissions.resource')" prop="resource"><el-input v-model="form.resource" /></el-form-item>
        <el-form-item :label="$t('permissions.action')" prop="action">
           <el-select v-model="form.action" class="w-full">
            <el-option v-for="action in Object.values(ActionType)" :key="action" :label="action" :value="action" />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('common.description')" prop="description"><el-input v-model="form.description" type="textarea" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog.visible = false">{{ $t("common.cancel") }}</el-button>
        <el-button type="primary" @click="submit">{{ $t("common.confirm") }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>
<style scoped></style>
