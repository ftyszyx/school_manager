<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import {
  getSchools,
  createSchool,
  updateSchool,
  deleteSchool
} from '@/apis/schools'
import type { School, SchoolCreateRequest, SchoolUpdateRequest } from '@/types/schools'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const schools = ref<School[]>([])
const loading = ref(true)
const showModal = ref(false)
const isEdit = ref(false)
const currentSchool = ref<SchoolCreateRequest | SchoolUpdateRequest>({ name: '' })
const currentSchoolId = ref<number | null>(null)
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const formRef = ref<FormInstance>()

const rules = reactive<FormRules>({
  name: [{ required: true, message: 'Name is required', trigger: 'blur' }]
})

const fetchSchools = async () => {
  loading.value = true
  try {
    const res = await getSchools({ page: page.value, page_size: pageSize.value })
    schools.value = res.list
    total.value = res.total
  } catch (error) {
    console.error(error)
  } finally {
    loading.value = false
  }
}

const handleAdd = () => {
  isEdit.value = false
  currentSchool.value = { name: '' }
  currentSchoolId.value = null
  showModal.value = true
}

const handleEdit = (school: School) => {
  isEdit.value = true
  currentSchoolId.value = school.id
  currentSchool.value = { name: school.name }
  showModal.value = true
}

const handleDelete = async (id: number) => {
  try {
    await ElMessageBox.confirm(
      t('common.delete_confirm', { name: schools.value.find(s => s.id === id)?.name || '' }),
      t('common.confirm'),
      { type: 'warning' }
    )
    await deleteSchool(id)
    ElMessage.success(t('common.deleted'))
    fetchSchools()
  } catch (error) {
    if (error !== 'cancel') {
      console.error(error)
    }
  }
}

const handleSubmit = async () => {
  const valid = await formRef.value?.validate()
  if (!valid) return

  try {
    if (isEdit.value && currentSchoolId.value) {
      await updateSchool(currentSchoolId.value, currentSchool.value as SchoolUpdateRequest)
      ElMessage.success(t('common.save'))
    } else {
      await createSchool(currentSchool.value as SchoolCreateRequest)
      ElMessage.success(t('common.created'))
    }
    showModal.value = false
    fetchSchools()
  } catch (error) {
    console.error(error)
  }
}

const handlePageChange = (p: number) => {
  page.value = p
  fetchSchools()
}

const handleSizeChange = (s: number) => {
  pageSize.value = s
  page.value = 1
  fetchSchools()
}

onMounted(() => {
  fetchSchools()
})
</script>

<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-end">
        <el-button type="success" @click="handleAdd">{{ $t("common.new") }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="schools" v-loading="loading" stripe size="large" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('common.name')" prop="name" min-width="160" />
        <el-table-column :label="$t('common.actions')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="handleEdit(row)">{{ $t("common.edit") }}</el-button>
            <el-button size="small" type="danger" @click="handleDelete(row.id)">{{ $t("common.delete") }}</el-button>
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

    <el-dialog v-model="showModal" :title="isEdit ? $t('common.edit') : $t('common.create')" width="520px">
      <el-form ref="formRef" :model="currentSchool" :rules="rules" label-width="140px">
        <el-form-item :label="$t('common.name')" prop="name">
          <el-input v-model="currentSchool.name" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showModal = false">{{ $t("common.cancel") }}</el-button>
        <el-button type="primary" @click="handleSubmit">{{ $t("common.confirm") }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped></style>
