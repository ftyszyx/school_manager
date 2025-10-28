<script setup lang="ts">
import { ref, onMounted, h } from 'vue'
import {
  getClasses,
  createClass,
  updateClass,
  deleteClass,
  getClass
} from '@/apis/classes'
import { getSchools } from '@/apis/schools'
import type { ClassInfo, ClassCreateRequest, ClassUpdateRequest } from '@/types/classes'
import type { School } from '@/types/schools'
import { NButton, NDataTable, NPopconfirm, NCard, NFlex, NModal, NForm, NFormItem, NInput, NSelect, NInputNumber } from 'naive-ui'

const classes = ref<ClassInfo[]>([])
const schools = ref<School[]>([])
const loading = ref(true)
const showModal = ref(false)
const isEdit = ref(false)
const currentClass = ref<Partial<ClassCreateRequest | ClassUpdateRequest>>({ name: '', grade: undefined, class: undefined, school_id: undefined })
const currentClassId = ref<number | null>(null)

const schoolOptions = ref<{ label: string; value: number }[]>([])

const columns = [
  { title: 'ID', key: 'id' },
  { title: 'Name', key: 'name' },
  { title: 'School ID', key: 'school_id' },
  { title: 'Grade', key: 'grade' },
  { title: 'Class', key: 'class' },
  { title: 'Status', key: 'status' },
  {
    title: 'Actions',
    key: 'actions',
    render(row: ClassInfo) {
      return h(NFlex, null, {
        default: () => [
          h(
            NButton,
            { size: 'small', onClick: () => handleEdit(row) },
            { default: () => 'Edit' }
          ),
          h(
            NPopconfirm,
            { onPositiveClick: () => handleDelete(row.id) },
            {
              trigger: () => h(NButton, { size: 'small', type: 'error' }, { default: () => 'Delete' }),
              default: () => 'Are you sure you want to delete this class?'
            }
          )
        ]
      })
    }
  }
]

const fetchClasses = async () => {
  loading.value = true
  try {
    const res = await getClasses({})
    classes.value = res.list
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
    schoolOptions.value = res.list.map((s) => ({ label: s.name, value: s.id }))
  } catch (error) {
    console.error(error)
  }
}

const handleAdd = () => {
  isEdit.value = false
  currentClass.value = { name: '', grade: undefined, class: undefined, school_id: undefined }
  showModal.value = true
}

const handleEdit = async (classInfo: ClassInfo) => {
  isEdit.value = true
  currentClassId.value = classInfo.id
  const res = await getClass(classInfo.id)
  currentClass.value = { 
    name: res.name, 
    grade: res.grade,
    class: res.class,
    school_id: res.school_id,
    status: res.status,
    password: res.password
  }
  showModal.value = true
}

const handleDelete = async (id: number) => {
  try {
    await deleteClass(id)
    fetchClasses()
  } catch (error) {
    console.error(error)
  }
}

const handleSubmit = async () => {
  try {
    if (isEdit.value && currentClassId.value) {
      await updateClass(currentClassId.value, currentClass.value as ClassUpdateRequest)
    } else {
      await createClass(currentClass.value as ClassCreateRequest)
    }
    showModal.value = false
    fetchClasses()
  } catch (error) {
    console.error(error)
  }
}

onMounted(() => {
  fetchClasses()
  fetchSchools()
})
</script>

<template>
  <NCard title="Classes Management">
    <NFlex justify="end" class="mb-4">
      <NButton @click="handleAdd">Add Class</NButton>
    </NFlex>
    <NDataTable :columns="columns" :data="classes" :loading="loading" />
    <NModal v-model:show="showModal" preset="card" style="width: 600px" :title="isEdit ? 'Edit Class' : 'Add Class'">
      <NForm @submit.prevent="handleSubmit">
        <NFormItem label="Name" required>
          <NInput v-model:value="currentClass.name" />
        </NFormItem>
        <NFormItem label="School" required>
          <NSelect v-model:value="currentClass.school_id" :options="schoolOptions" />
        </NFormItem>
        <NFormItem label="Grade" required>
          <NInputNumber v-model:value="currentClass.grade" />
        </NFormItem>
        <NFormItem label="Class" required>
          <NInputNumber v-model:value="currentClass.class" />
        </NFormItem>
         <NFormItem label="Status">
          <NInputNumber v-model:value="currentClass.status" />
        </NFormItem>
        <NFormItem label="Password">
          <NInput v-model:value="currentClass.password" type="password" />
        </NFormItem>
        <NFlex justify="end">
          <NButton @click="showModal = false">Cancel</NButton>
          <NButton type="primary" attr-type="submit">Submit</NButton>
        </NFlex>
      </NForm>
    </NModal>
  </NCard>
</template>
