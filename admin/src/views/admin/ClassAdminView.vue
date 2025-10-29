<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import {
  getClasses,
  createClass,
  updateClass,
  deleteClass,
  createClassesBulk
} from '@/apis/classes'
import { getSchools } from '@/apis/schools'
import type { ClassInfo, ClassCreateRequest, ClassUpdateRequest, ClassUserInfo } from '@/types/classes'
import type { School } from '@/types/schools'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { useI18n } from 'vue-i18n'
import { useClipboard } from '@vueuse/core'

const { t } = useI18n()
const { copy, copied } = useClipboard()
const classes = ref<ClassInfo[]>([])
const schools = ref<School[]>([])
const loading = ref(true)
const showModal = ref(false)
const isEdit = ref(false)
const query = reactive({ name: "" as string | undefined, school_id: undefined as number | undefined, grade: undefined as number | undefined, status: undefined as number | undefined });
const currentClass = ref<Partial<ClassCreateRequest | ClassUpdateRequest>>({})
const currentClassId = ref<number | null>(null)
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)
const formRef = ref<FormInstance>()

const bulkAddDialog = reactive({
  visible: false,
  schoolId: undefined as number | undefined,
  grades: [{  classCount: 4 }]
})

const statusOptions = [
  { label: '已放学', value: 0 },
  { label: '上课中', value: 1 },
  { label: '放学中', value: 2 }
]

const rules = reactive<FormRules>({
  name: [{ required: true, message: 'Name is required', trigger: 'blur' }],
  school_id: [{ required: true, message: 'School is required', trigger: 'change' }],
  class: [{ required: true, message: 'Class is required', trigger: 'blur' }]
})

const schoolNameMap = computed(() => {
  return schools.value.reduce((map, school) => {
    map[school.id] = school.name
    return map
  }, {} as Record<number, string>)
})

const statusMap = computed(() => {
  return statusOptions.reduce((map, option) => {
    map[option.value] = option.label
    return map
  }, {} as Record<number, string>)
})

const statusTypeMap = {
  0: 'info',
  1: 'success',
  2: 'warning'
} as const

const fetchClasses = async () => {
  loading.value = true
  try {
    const res = await getClasses({ 
      page: page.value, 
      page_size: pageSize.value,
      school_id: query.school_id,
      grade: query.grade,
      name: query.name,
      status: query.status,
     })
    classes.value = res.list
    total.value = res.total
  } catch (error) {
    console.error(error)
  } finally {
    loading.value = false
  }
}

const fetchSchools = async () => {
  try {
    const res = await getSchools({ page_size: 1000 }) // Fetch all schools
    schools.value = res.list
  } catch (error) {
    console.error(error)
  }
}

const handleAdd = () => {
  isEdit.value = false
  currentClass.value = { name: '', grade: undefined, class: undefined, school_id: undefined, status: 0, password: '' }
  currentClassId.value = null
  showModal.value = true
}

const resetFilters = () => {
  query.name = ""
  query.school_id = undefined
  query.grade = undefined
  query.status = undefined
  page.value = 1
  fetchClasses()
}

const openBulkAdd = () => {
  bulkAddDialog.grades = [{  classCount: 4 }]
  bulkAddDialog.schoolId = undefined
  bulkAddDialog.visible = true
}

const addGrade = () => {
  bulkAddDialog.grades.push({  classCount: 4 })
}

const removeGrade = (index: number) => {
  bulkAddDialog.grades.splice(index, 1)
}

const handleBulkSubmit = async () => {
  if (!bulkAddDialog.schoolId) {
    ElMessage.error(t('classes.please_select_school'))
    return
  }
  const classesToCreate: ClassCreateRequest[] = []
  for (let i = 1; i <= bulkAddDialog.grades.length; i++) {
    const gradinfo = bulkAddDialog.grades[i - 1]
    for (let j = 1; j <= gradinfo.classCount; j++) {
      classesToCreate.push({
        school_id: bulkAddDialog.schoolId,
        grade: i,
        class: j,
        name: `${i}年级${j}班`,
        password: generatePassword(),
      })
    }
  }

  try {
    await createClassesBulk({ classes: classesToCreate })
    ElMessage.success(t('common.created'))
    bulkAddDialog.visible = false
    fetchClasses()
  } catch (error) {
    console.error(error)
  }
}


const handleEdit = (classInfo: ClassInfo) => {
  isEdit.value = true
  currentClassId.value = classInfo.id
  currentClass.value = { ...classInfo }
  showModal.value = true
}

const generatePasswordForCurrnet = () => {
  const password=generatePassword()
  if (currentClass.value) {
    currentClass.value.password = password
  }
}

const generatePassword = () => {
  // const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
  const chars = '0123456789'
  let password = ''
  for (let i = 0; i < 6; i++) {
    password += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  return password
}

const handleCopy = (text: string) => {
  copy(text)
  if (copied.value) {
    ElMessage.success(t('common.copied'))
  }
}

const handleDelete = async (id: number) => {
  try {
    await ElMessageBox.confirm(
      t('common.delete_confirm', { name: classes.value.find(c => c.id === id)?.name || '' }),
      t('common.confirm'),
      { type: 'warning' }
    )
    await deleteClass(id)
    ElMessage.success(t('common.deleted'))
    fetchClasses()
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
    if (isEdit.value && currentClassId.value) {
      await updateClass(currentClassId.value, currentClass.value as ClassUpdateRequest)
      ElMessage.success(t('common.save'))
    } else {
      await createClass(currentClass.value as ClassCreateRequest)
      ElMessage.success(t('common.created'))
    }
    showModal.value = false
    fetchClasses()
  } catch (error) {
    console.error(error)
  }
}

const handlePageChange = (p: number) => {
  page.value = p
  fetchClasses()
}

const handleSizeChange = (s: number) => {
  pageSize.value = s
  page.value = 1
  fetchClasses()
}

onMounted(() => {
  fetchClasses()
  fetchSchools()
})
</script>

<template>
  <div class="space-y-4">
    <el-card shadow="hover">
      <div class="flex items-center justify-start gap-2">
         <el-select v-model="query.school_id" :placeholder="$t('schools.school')" clearable class="w-48">
          <el-option v-for="school in schools" :key="school.id" :label="school.name" :value="school.id" />
        </el-select>
        <el-input-number v-model="query.grade" :placeholder="$t('classes.grade')" clearable class="w-48" />
        <el-select v-model="query.status" :placeholder="$t('common.status')" clearable class="w-48">
          <el-option v-for="option in statusOptions" :key="option.value" :label="option.label" :value="option.value" />
        </el-select>
        <el-input v-model="query.name" :placeholder="$t('common.search_by_name')" clearable class="w-48" />
        <el-button type="primary" @click="fetchClasses">{{ $t("common.search") }}</el-button>
        <el-button @click="resetFilters">{{ $t("common.reset") }}</el-button>
        <el-button type="primary" @click="openBulkAdd">{{ $t("classes.bulk_add") }}</el-button>
        <el-button type="success" @click="handleAdd">{{ $t("common.new") }}</el-button>
      </div>
    </el-card>

    <el-card shadow="never">
      <el-table :data="classes" v-loading="loading" stripe size="large" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column :label="$t('common.name')" prop="name" min-width="160" />
        <el-table-column :label="$t('schools.school')" min-width="160">
          <template #default="{ row }">{{ schoolNameMap[row.school_id] }}</template>
        </el-table-column>
        <el-table-column :label="$t('classes.grade')" prop="grade" width="100" />
        <el-table-column :label="$t('classes.class')" prop="class" width="100" />
        <el-table-column :label="$t('common.status')" prop="status" width="120">
           <template #default="{ row }">
            <el-tag :type="statusTypeMap[row.status as keyof typeof statusTypeMap]">{{ statusMap[row.status as keyof typeof statusMap] }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('classes.teachers')" min-width="150">
          <template #default="{ row }">
            {{ row.teacher_infos.map((t: ClassUserInfo) => t.user_name).join(', ') }}
          </template>
        </el-table-column>
        <el-table-column :label="$t('classes.password')" prop="password" min-width="150">
          <template #default="{ row }">
            <div class="flex items-center">
              <span>{{ row.password }}</span>
              <el-button link type="primary" @click="handleCopy(row.password)" class="ml-2">
                {{ $t('common.copy') }}
              </el-button>
            </div>
          </template>
        </el-table-column>
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
      <el-form ref="formRef" :model="currentClass" :rules="rules" label-width="140px">
        <el-form-item :label="$t('common.name')" prop="name">
          <el-input v-model="currentClass.name" />
        </el-form-item>
        <el-form-item :label="$t('schools.school')" prop="school_id">
          <el-select v-model="currentClass.school_id" class="w-full">
            <el-option v-for="school in schools" :key="school.id" :label="school.name" :value="school.id" />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('classes.grade')" prop="grade">
          <el-input-number v-model="currentClass.grade" :min="1" />
        </el-form-item>
        <el-form-item :label="$t('classes.class')" prop="class">
          <el-input-number v-model="currentClass.class" :min="1" />
        </el-form-item>
        <el-form-item :label="$t('common.status')" prop="status">
          <el-select v-model="currentClass.status" class="w-full">
            <el-option v-for="option in statusOptions" :key="option.value" :label="option.label" :value="option.value" />
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('classes.password')" prop="password">
          <el-input v-model="currentClass.password" type="password" show-password >
            <template #append>
              <el-button @click="generatePasswordForCurrnet">{{ $t('common.generate') }}</el-button>
            </template>
          </el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showModal = false">{{ $t("common.cancel") }}</el-button>
        <el-button type="primary" @click="handleSubmit">{{ $t("common.confirm") }}</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="bulkAddDialog.visible" :title="$t('classes.bulk_add')" width="720px">
      <el-form label-width="140px">
        <el-form-item :label="$t('schools.school')" required>
          <el-select v-model="bulkAddDialog.schoolId" class="w-full">
            <el-option v-for="school in schools" :key="school.id" :label="school.name" :value="school.id" />
          </el-select>
        </el-form-item>

        <div v-for="(item, index) in bulkAddDialog.grades" :key="index" class="bg-slate-50 p-4 rounded-md mb-4">
          <el-form-item :label="`${$t('classes.grade')} ${index + 1}`">
            <div class="flex items-center w-full">
              <span class="mr-4 text-gray-500 flex-shrink-0">{{ $t('classes.class_count') }}</span>
              <el-input-number v-model="item.classCount" :min="1" class="mr-auto" />
              <el-button type="danger" @click="removeGrade(index)">{{ $t('common.delete') }}</el-button>
            </div>
          </el-form-item>
        </div>

        <el-form-item label=" ">
          <el-button @click="addGrade">{{ $t('classes.add_grade') }}</el-button>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="bulkAddDialog.visible = false">{{ $t("common.cancel") }}</el-button>
        <el-button type="primary" @click="handleBulkSubmit">{{ $t("common.confirm") }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped></style>
