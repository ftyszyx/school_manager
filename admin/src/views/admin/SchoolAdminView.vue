<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  getSchools,
  createSchool,
  updateSchool,
  deleteSchool,
  getSchool
} from '@/apis/schools'
import type { School, SchoolCreateRequest, SchoolUpdateRequest } from '@/types/schools'
import { NButton, NDataTable, NPopconfirm, NCard, NFlex, NModal, NForm, NFormItem, NInput } from 'naive-ui'

const schools = ref<School[]>([])
const loading = ref(true)
const showModal = ref(false)
const isEdit = ref(false)
const currentSchool = ref<SchoolCreateRequest | SchoolUpdateRequest>({ name: '' })
const currentSchoolId = ref<number | null>(null)

const columns = [
  { title: 'ID', key: 'id' },
  { title: 'Name', key: 'name' },
  {
    title: 'Actions',
    key: 'actions',
    render(row: School) {
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
              default: () => 'Are you sure you want to delete this school?'
            }
          )
        ]
      })
    }
  }
]

const fetchSchools = async () => {
  loading.value = true
  try {
    const res = await getSchools({})
    schools.value = res.list
  } catch (error) {
    console.error(error)
  } finally {
    loading.value = false
  }
}

const handleAdd = () => {
  isEdit.value = false
  currentSchool.value = { name: '' }
  showModal.value = true
}

const handleEdit = async (school: School) => {
  isEdit.value = true
  currentSchoolId.value = school.id
  const res = await getSchool(school.id)
  currentSchool.value = { name: res.name }
  showModal.value = true
}

const handleDelete = async (id: number) => {
  try {
    await deleteSchool(id)
    fetchSchools()
  } catch (error) {
    console.error(error)
  }
}

const handleSubmit = async () => {
  try {
    if (isEdit.value && currentSchoolId.value) {
      await updateSchool(currentSchoolId.value, currentSchool.value as SchoolUpdateRequest)
    } else {
      await createSchool(currentSchool.value as SchoolCreateRequest)
    }
    showModal.value = false
    fetchSchools()
  } catch (error) {
    console.error(error)
  }
}

onMounted(() => {
  fetchSchools()
})
</script>

<template>
  <NCard title="Schools Management">
    <NFlex justify="end" class="mb-4">
      <NButton @click="handleAdd">Add School</NButton>
    </NFlex>
    <NDataTable :columns="columns" :data="schools" :loading="loading" />
    <NModal v-model:show="showModal" preset="card" style="width: 600px" :title="isEdit ? 'Edit School' : 'Add School'">
      <NForm @submit.prevent="handleSubmit">
        <NFormItem label="Name" required>
          <NInput v-model:value="currentSchool.name" />
        </NFormItem>
        <NFlex justify="end">
          <NButton @click="showModal = false">Cancel</NButton>
          <NButton type="primary" attr-type="submit">Submit</NButton>
        </NFlex>
      </NForm>
    </NModal>
  </NCard>
</template>
