<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  getSchools,
  createSchool,
  updateSchool,
  deleteSchool,
  getSchoolClassStatusConfigs,
  updateSchoolClassStatusConfigs
} from '@/apis/schools'
import type {
  School,
  SchoolClassStatusConfigItem,
  SchoolCreateRequest,
  SchoolUpdateRequest,
  VantTagType
} from '@/types/schools'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const router = useRouter()
const schools = ref<School[]>([])
const loading = ref(true)
const showModal = ref(false)
const isEdit = ref(false)
const currentSchool = ref<SchoolCreateRequest | SchoolUpdateRequest>({ name: '', password: '' })
const currentSchoolId = ref<number | null>(null)
const currentStatusConfigs = ref<SchoolClassStatusConfigItem[]>([])
const schoolStatusConfigsMap = ref<Record<number, SchoolClassStatusConfigItem[]>>({})
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const formRef = ref<FormInstance>()

const rules = reactive<FormRules>({
  name: [{ required: true, message: 'Name is required', trigger: 'blur' }],
  password: [
    {
      required: true,
      message: 'Password is required for new schools',
      trigger: 'blur'
    }
  ]
})

const fetchSchools = async () => {
  loading.value = true
  try {
    const res = await getSchools({ page: page.value, page_size: pageSize.value })
    schools.value = res.list
    total.value = res.total


    const nextMap: Record<number, SchoolClassStatusConfigItem[]> = {}
    await Promise.all(
      (res.list || []).map(async (s) => {
        try {
          nextMap[s.id] = await getSchoolClassStatusConfigs(s.id)
        } catch (e) {
          nextMap[s.id] = []
        }
      })
    )
    schoolStatusConfigsMap.value = nextMap
  } catch (error) {
    console.error(error)
  } finally {
    loading.value = false
  }
}

const createDefaultStatusConfigs = (): SchoolClassStatusConfigItem[] => {
  const base: Array<{ status: number; sort_order: number }> = [
    { status: 0, sort_order: 0 },
    { status: 1, sort_order: 1 },
    { status: 2, sort_order: 2 }
  ]
  return base.map((b) => ({
    status: b.status,
    label: '',
    type: 'default' as VantTagType,
    sort_order: b.sort_order
  }))
}

const mapElTagType = (t: VantTagType) => {
  if (t === 'success') return 'success'
  if (t === 'warning') return 'warning'
  if (t === 'danger') return 'danger'
  if (t === 'default') return 'info'
  return 'primary'
}

const handleAdd = () => {
  isEdit.value = false
  currentSchool.value = { name: '', password: '' }
  currentSchoolId.value = null
  currentStatusConfigs.value = createDefaultStatusConfigs()
  showModal.value = true
}

const handleEdit = (school: School) => {
  isEdit.value = true
  currentSchoolId.value = school.id
  currentSchool.value = { name: school.name, password: school.password }
  currentStatusConfigs.value = (schoolStatusConfigsMap.value[school.id] || []).length
    ? (schoolStatusConfigsMap.value[school.id] || []).slice().sort((a, b) => a.sort_order - b.sort_order)
    : createDefaultStatusConfigs()
  showModal.value = true
}

const goToScreen = (schoolId: number) => {
	router.push(`/screen/${schoolId}`)
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
    const payload = { ...currentSchool.value }
    // If password is empty on edit, don't send it
    if (isEdit.value && !payload.password) {
      delete payload.password
    }

    if (isEdit.value && currentSchoolId.value) {
      await updateSchool(currentSchoolId.value, payload as SchoolUpdateRequest)
      await updateSchoolClassStatusConfigs(currentSchoolId.value, { configs: currentStatusConfigs.value })
      ElMessage.success(t('common.save'))
    } else {
      const created = await createSchool(payload as SchoolCreateRequest)
      await updateSchoolClassStatusConfigs(created.id, { configs: currentStatusConfigs.value })
      ElMessage.success(t('common.created'))
    }
    showModal.value = false
    fetchSchools()
  } catch (error) {
    console.error(error)
  }
}

const copyPassword = (password: string) => {
  navigator.clipboard.writeText(password).then(() => {
    ElMessage.success(t('common.copied'))
  })
}

const generatePassword = () => {
  const chars = '0123456789'
  let pass = ''
  for (let i = 0; i < 6; i++) {
    pass += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  currentSchool.value.password = pass
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
        <el-table-column :label="$t('common.password')" prop="password" min-width="200">
			<template #default="{ row }">
				<div class="flex items-center">
					<span>{{ row.password }}</span>
					<el-button
						type="text"
						icon="CopyDocument"
						@click="copyPassword(row.password)"
						class="ml-2"
					></el-button>
				</div>
			</template>
		</el-table-column>
		<el-table-column label="班级状态" min-width="260">
			<template #default="{ row }">
				<div class="flex flex-wrap gap-2">
					<template v-if="(schoolStatusConfigsMap[row.id] || []).length">
						<el-tag
							v-for="cfg in (schoolStatusConfigsMap[row.id] || []).slice().sort((a, b) => a.sort_order - b.sort_order)"
							:key="cfg.status"
							:type="mapElTagType(cfg.type)"
						>
							{{ cfg.status }}: {{ cfg.label || '未配置' }}
						</el-tag>
					</template>
					<el-tag v-else type="info">未配置</el-tag>
				</div>
			</template>
		</el-table-column>
        <el-table-column :label="$t('common.actions')" width="260" fixed="right">
          <template #default="{ row }">
            <div class="flex flex-wrap gap-2 justify-end">
              <el-button size="small" type="primary" @click="goToScreen(row.id)">大屏</el-button>
              <el-button size="small" @click="handleEdit(row)">{{ $t("common.edit") }}</el-button>
              <el-button size="small" type="danger" @click="handleDelete(row.id)">{{ $t("common.delete") }}</el-button>
            </div>
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
        <el-form-item :label="$t('common.password')" prop="password">
          <el-input v-model="currentSchool.password" show-password>
            <template #append>
              <el-button @click="generatePassword">{{ $t('common.generate') }}</el-button>
            </template>
          </el-input>
        </el-form-item>
		<el-form-item label="班级状态配置">
			<div class="w-full space-y-2">
				<div
					v-for="(cfg) in currentStatusConfigs"
					:key="cfg.status"
					class="flex items-center gap-2"
				>
					<el-tag type="info" style="min-width: 48px; text-align: center">{{ cfg.status }}</el-tag>
					<el-input v-model="cfg.label" placeholder="显示名" style="width: 180px" />
					<el-select v-model="cfg.type" placeholder="type" style="width: 140px">
						<el-option label="default" value="default">
							<el-tag type="info">default</el-tag>
						</el-option>
						<el-option label="primary" value="primary">
							<el-tag type="primary">primary</el-tag>
						</el-option>
						<el-option label="success" value="success">
							<el-tag type="success">success</el-tag>
						</el-option>
						<el-option label="warning" value="warning">
							<el-tag type="warning">warning</el-tag>
						</el-option>
						<el-option label="danger" value="danger">
							<el-tag type="danger">danger</el-tag>
						</el-option>
					</el-select>
				</div>
			</div>
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
