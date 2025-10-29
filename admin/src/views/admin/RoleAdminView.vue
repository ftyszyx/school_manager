<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { getRoles, createRole, updateRole, deleteRole } from '@/apis/roles'
import { getPermissions } from '@/apis/permissions'
import { useI18n } from 'vue-i18n'
import type { Role, Permission } from '@/types'

const rows = ref<Role[]>([])
const allPermissions = ref<Permission[]>([])
const selectedIds = ref<number[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const { t } = useI18n()

const query = reactive({ name: '' as string | undefined })

async function reload() {
  const data = await getRoles({ page: page.value, page_size: pageSize.value, name: query.name || undefined })
  rows.value = data.list
  total.value = data.total
}

async function fetchAllPermissions() {
  const data = await getPermissions({ page: 1, page_size: 1000 })
  allPermissions.value = data.list
}

function resetFilters() {
  query.name = ''
  page.value = 1
  reload()
}
function onSelChange(arr: Role[]) {
  selectedIds.value = arr.map((it) => it.id)
}
function handlePageChange(p: number) {
  page.value = p
  reload()
}
function handleSizeChange(s: number) {
  pageSize.value = s
  page.value = 1
  reload()
}

const dialog = reactive({ visible: false, mode: 'create' as 'create' | 'edit', editingId: undefined as number | undefined })
const formRef = ref<FormInstance>()
const form = reactive<{ name: string; description?: string | null; permission_ids: number[] }>({ name: '', description: '', permission_ids: [] })
const rules = reactive<FormRules>({ name: [{ required: true, message: 'Name required' }] })

const permissionTransferData = computed(() => {
  return allPermissions.value.map(p => ({
    key: p.id,
    label: `${p.name} (${p.resource}:${p.action})`,
    disabled: false
  }))
})

function openCreate() {
  dialog.mode = 'create'
  dialog.editingId = undefined
  form.name = ''
  form.description = ''
  form.permission_ids = []
  dialog.visible = true
}
function openEdit(row: Role) {
  dialog.mode = 'edit'
  dialog.editingId = row.id
  form.name = row.name
  form.description = row.description || ''
  form.permission_ids = row.permission_infos?.map(p => p.id) || []
  dialog.visible = true
}

async function submit() {
  const valid = await formRef.value?.validate()
  if (!valid) {
    ElMessage.error(t('common.please_check_form') as string)
    return
  }
  const payload = {
    name: form.name,
    description: form.description || undefined,
    permission_ids: form.permission_ids
  }
  if (dialog.mode === 'create') {
    await createRole(payload)
    ElMessage.success(t('common.created') as string)
  } else if (dialog.editingId != null) {
    await updateRole(dialog.editingId, payload)
    ElMessage.success(t('common.save') as string)
  }
  dialog.visible = false
  await reload()
}

async function del(id: number) {
  await ElMessageBox.confirm(t('common.delete_confirm', { name: rows.value.find((it: Role) => it.id === id)?.name || '' }), t('common.confirm'), { type: 'warning' })
  await deleteRole(id)
  ElMessage.success(t('common.deleted') as string)
  reload()
}

onMounted(() => {
  reload()
  fetchAllPermissions()
})
</script>

<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center gap-2">
        <el-input v-model="query.name" :placeholder="$t('common.search_by_name')" clearable class="w-64" />
        <el-button type="primary" @click="reload">{{ $t('common.search') }}</el-button>
        <el-button @click="resetFilters">{{ $t('common.reset') }}</el-button>
        <el-button type="success" @click="openCreate">{{ $t('common.new') }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="rows" stripe size="large" style="width: 100%" @selection-change="onSelChange">
        <el-table-column type="selection" width="50" />
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('common.name')" min-width="160">
          <template #default="{ row }">
            <div class="flex items-center gap-2">
              <span class="text-gray-800 break-all">{{ row.name }}</span>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.description')" min-width="200">
          <template #default="{ row }">{{ row.description }}</template>
        </el-table-column>
        <el-table-column :label="$t('menu.permissions')" min-width="200">
          <template #default="{ row }">{{ row.permission_infos?.map((it:Permission) => it.name).join(', ') }}</template>
        </el-table-column>
        <el-table-column :label="$t('common.actions')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button size="small" type="danger" @click="del(row.id)">{{ $t('common.delete') }}</el-button>
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

    <el-dialog v-model="dialog.visible" :title="dialog.mode === 'create' ? $t('common.create') : $t('common.edit')" width="820px">
      <el-form label-width="140px" ref="formRef" :model="form" :rules="rules">
        <el-form-item :label="$t('common.name')" prop="name"><el-input v-model="form.name" /></el-form-item>
        <el-form-item :label="$t('common.description')" prop="description"><el-input v-model="form.description" type="textarea" /></el-form-item>
        <el-form-item :label="$t('menu.permissions')" prop="permission_ids">
          <el-transfer
            v-model="form.permission_ids"
            :data="permissionTransferData"
            :titles="[$t('common.available'), $t('common.assigned')]"
            filterable
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog.visible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit">{{ $t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped></style>


