<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref } from 'vue'
import { useRoute } from 'vue-router'
import { getClassesBySchool } from '@/apis/classes'
import type { ClassInfo } from '@/types/classes'

interface ScreenClass extends ClassInfo {
  id: number
  grade: number
  class: number
  status: number
}

const route = useRoute()
const schoolId = Number(route.params.schoolId)

const loading = ref(true)
const classes = ref<ScreenClass[]>([])
const schoolName = ref('')
const lastUpdate = ref<Date | null>(null)
let socket: WebSocket | null = null

const statusMap: Record<number, { text: string; color: string }> = {
  0: { text: '未放学', color: '#0066FF' },
  1: { text: '放学中', color: '#FF6600' },
  2: { text: '已放学', color: '#00C853' },
}

const nowTime = ref(new Date())
const formattedNow = computed(() => nowTime.value.toLocaleString())
const formattedLastUpdate = computed(() => (lastUpdate.value ? lastUpdate.value.toLocaleString() : ''))
let clockTimer: number | null = null

const grades = computed(() => {
  const gradeSet = new Set<number>()
  classes.value.forEach((cls) => gradeSet.add(Number(cls.grade)))
  return Array.from(gradeSet).sort((a, b) => a - b)
})

const maxClassCount = computed(() => {
  return classes.value.reduce((max, cls) => Math.max(max, Number(cls.class)), 0)
})

const getStatusLabel = (status: number) => {
  return statusMap[status]?.text ?? '未知'
}

const getStatusColor = (status: number) => {
  return statusMap[status]?.color ?? '#999999'
}

const tableRows = computed(() => {
  return grades.value.map((grade) => {
    const row: { grade: number; classes: ScreenClass[] } = {
      grade,
      classes: [],
    }
    for (let clsNo = 1; clsNo <= maxClassCount.value; clsNo++) {
      const cls = classes.value.find((c) => Number(c.grade) === grade && Number(c.class) === clsNo)
      if (cls) {
        row.classes.push(cls)
      } else {
        row.classes.push({
          id: -1,
          grade,
          class: clsNo,
          status: -1,
          school_id: schoolId,
          school_name: schoolName.value,
          name: '',
          teacher_infos: [],
        } as ScreenClass)
      }
    }
    return row
  })
})

const connectWebSocket = () => {
  if (!import.meta.env.VITE_BASE_URL) return
  try {
    const base = import.meta.env.VITE_BASE_URL.replace(/^http/, 'ws')
    const url = `${base}/ws/school/${schoolId}`
    socket = new WebSocket(url)
    socket.onopen = () => console.log('screen ws connected')
    socket.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data)
        if (payload.event_type === 'CLASS_STATUS_UPDATE' && payload.data) {
          const updated = payload.data
          classes.value = classes.value.map((cls) =>
            cls.id === updated.id ? { ...cls, status: Number(updated.status) } : cls,
          )
          lastUpdate.value = new Date()
        }
      } catch (err) {
        console.warn('Invalid ws payload', err)
      }
    }
    socket.onerror = (err) => console.error('screen ws error', err)
    socket.onclose = () => console.log('screen ws closed')
  } catch (err) {
    console.error('Failed to open websocket', err)
  }
}

const loadData = async () => {
  loading.value = true
  try {
    const list = await getClassesBySchool(schoolId)
    const normalized: ScreenClass[] = list.map((c: any) => ({
      ...c,
      id: Number(c.id ?? c.class_id ?? -1),
      grade: Number(c.grade ?? 0),
      class: Number(c.class ?? c.class_num ?? 0),
      status: Number(c.status ?? 0),
    }))
    classes.value = normalized
    schoolName.value = normalized[0]?.school_name ?? '学校'
    lastUpdate.value = new Date()
  } catch (err) {
    console.error('Failed to load classes', err)
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  if (!schoolId) {
    console.error('Invalid school id in route')
    return
  }
  await loadData()
  connectWebSocket()
  clockTimer = window.setInterval(() => {
    nowTime.value = new Date()
  }, 1000)
})

onBeforeUnmount(() => {
  if (socket) {
    socket.close()
    socket = null
  }
  if (clockTimer) {
    clearInterval(clockTimer)
    clockTimer = null
  }
})
</script>

<template>
  <div class="screen-container">
    <div class="screen-header">
      <div class="title">{{ schoolName }}</div>
      <div class="timestamp">
        <div class="clock">{{ formattedNow }}</div>
        <div class="update" v-if="formattedLastUpdate">上次更新：{{ formattedLastUpdate }}</div>
      </div>
    </div>

    <div v-if="loading" class="screen-loading">加载中...</div>

    <div v-else class="screen-table">
      <div class="table-header">
        <div class="grade-cell"></div>
        <div class="class-cell" v-for="cls in maxClassCount" :key="cls">{{ cls }}班</div>
      </div>
      <div class="table-row" v-for="row in tableRows" :key="row.grade">
        <div class="grade-cell">{{ row.grade }}年级</div>
        <div
          class="status-cell"
          v-for="cls in row.classes"
          :key="`${row.grade}-${cls.class}`"
          :style="{ color: getStatusColor(cls.status) }"
        >
          {{ getStatusLabel(cls.status) }}
        </div>
      </div>
    </div>

    <div class="screen-legend">
      <div class="legend-item" v-for="(info, key) in statusMap" :key="key">
        <span class="legend-color" :style="{ backgroundColor: info.color }"></span>
        <span class="legend-text">{{ info.text }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.screen-container {
  min-height: 100vh;
  width: 100%;
  background: linear-gradient(180deg, #0f4c75 0%, #1b262c 100%);
  color: #ffffff;
  font-family: 'Microsoft YaHei', 'PingFang SC', 'Helvetica Neue', sans-serif;
  padding: 32px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.screen-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.screen-header .title {
  font-size: 48px;
  font-weight: 700;
  letter-spacing: 4px;
}

.screen-header .timestamp {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  font-size: 28px;
  color: rgba(255, 255, 255, 0.9);
  gap: 6px;
}

.screen-header .timestamp .update {
  font-size: 20px;
  color: rgba(255, 255, 255, 0.75);
}

.screen-loading {
  font-size: 28px;
  text-align: center;
  margin-top: 60px;
}

.screen-table {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.table-header, .table-row {
  display: grid;
  grid-template-columns: 160px repeat(auto-fit, minmax(160px, 1fr));
  gap: 12px;
  align-items: center;
}

.grade-cell {
  font-size: 28px;
  font-weight: 600;
  text-align: center;
  background: rgba(255, 255, 255, 0.1);
  padding: 12px;
  border-radius: 12px;
}

.class-cell, .status-cell {
  font-size: 28px;
  text-align: center;
  background: rgba(255, 255, 255, 0.1);
  padding: 12px;
  border-radius: 12px;
}

.status-cell {
  font-weight: 600;
}

.screen-legend {
  display: flex;
  justify-content: center;
  gap: 48px;
  margin-top: 16px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 24px;
}

.legend-color {
  width: 32px;
  height: 32px;
  border-radius: 8px;
}
</style>
