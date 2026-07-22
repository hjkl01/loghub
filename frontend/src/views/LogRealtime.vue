<template>
  <div>
    <div style="margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center">
      <h2>实时日志</h2>
      <div>
        <el-button :type="connected ? 'success' : 'danger'" size="small" @click="toggleConnection">
          {{ connected ? '已连接' : '未连接' }}
        </el-button>
        <el-button :disabled="!connected" @click="paused = !paused">
          {{ paused ? '继续滚动' : '暂停' }}
        </el-button>
        <el-button :disabled="logs.length === 0" @click="logs = []">清屏</el-button>
      </div>
    </div>

    <el-card style="margin-bottom: 16px">
      <el-form :inline="true" :model="filters">
        <el-form-item label="级别">
          <el-select v-model="filters.levels" placeholder="全部级别" multiple clearable style="width: 200px">
            <el-option label="DEBUG" value="DEBUG" />
            <el-option label="INFO" value="INFO" />
            <el-option label="WARN" value="WARN" />
            <el-option label="ERROR" value="ERROR" />
          </el-select>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card>
      <div ref="logContainer" style="height: 500px; overflow-y: auto; background: #1e1e1e; color: #d4d4d4; padding: 12px; font-family: 'Courier New', monospace; font-size: 13px; line-height: 1.6">
        <div v-if="logs.length === 0" style="color: #666; text-align: center; padding-top: 200px">
          等待日志...
        </div>
        <div v-for="(log, idx) in logs" :key="idx" style="display: flex; gap: 8px; cursor: pointer" @click="showDetail(log)">
          <span style="color: #888; min-width: 160px">{{ formatTime(log.time) }}</span>
          <span :style="{ color: levelColor(log.level), minWidth: '50px', fontWeight: 'bold' }">{{ log.level }}</span>
          <span style="color: #b0b0b0; min-width: 100px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">{{ log.service }}</span>
          <span v-if="log.file_name" style="color: #6a9955; min-width: 80px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">{{ log.file_name }}:{{ log.line_number }}</span>
          <span>{{ log.message }}</span>
        </div>
      </div>
    </el-card>

    <el-dialog v-model="detailVisible" title="日志详情" width="700px">
      <el-descriptions :column="2" border>
        <el-descriptions-item label="日志时间">{{ formatTime(detailLog?.time) }}</el-descriptions-item>
        <el-descriptions-item label="入库时间">{{ formatTime(detailLog?.ingest_time) }}</el-descriptions-item>
        <el-descriptions-item label="级别">
          <el-tag :type="levelTag(detailLog?.level)" size="small">{{ detailLog?.level }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="服务">{{ detailLog?.service }}</el-descriptions-item>
        <el-descriptions-item label="文件名">{{ detailLog?.file_name || '-' }}</el-descriptions-item>
        <el-descriptions-item label="函数名">{{ detailLog?.function_name || '-' }}</el-descriptions-item>
        <el-descriptions-item label="行号">{{ detailLog?.line_number || '-' }}</el-descriptions-item>
      </el-descriptions>
      <div style="margin-top: 16px">
        <div style="font-weight: bold; margin-bottom: 8px">消息内容</div>
        <pre style="white-space: pre-wrap; word-break: break-all; background: #f5f7fa; padding: 12px; border-radius: 4px; max-height: 300px; overflow-y: auto">{{ detailLog?.message }}</pre>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, watch, nextTick } from 'vue'

const logs = ref<any[]>([])
const connected = ref(false)
const paused = ref(false)
const logContainer = ref<HTMLElement>()
const detailVisible = ref(false)
const detailLog = ref<any>(null)

const filters = reactive({
  levels: [] as string[],
})

let ws: WebSocket | null = null
let reconnectTimer: any = null

function formatTime(t: string) {
  if (!t) return ''
  return t.replace('T', ' ').substring(0, 19)
}

function levelColor(level: string) {
  if (level === 'ERROR') return '#f56c6c'
  if (level === 'WARN') return '#e6a23c'
  if (level === 'DEBUG') return '#909399'
  return '#67c23a'
}

function levelTag(level: string) {
  if (level === 'ERROR') return 'danger'
  if (level === 'WARN') return 'warning'
  if (level === 'DEBUG') return 'info'
  return 'success'
}

function showDetail(log: any) {
  detailLog.value = log
  detailVisible.value = true
}

function connect() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  ws = new WebSocket(`${protocol}//${window.location.host}/api/logs/ws`)

  ws.onopen = () => {
    connected.value = true
    sendFilter()
  }

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data)
      if (msg.type === 'log') {
        logs.value.push(msg.data)
        if (logs.value.length > 1000) {
          logs.value = logs.value.slice(-500)
        }
        if (!paused.value) {
          nextTick(() => {
            if (logContainer.value) {
              logContainer.value.scrollTop = logContainer.value.scrollHeight
            }
          })
        }
      }
    } catch {}
  }

  ws.onclose = () => {
    connected.value = false
    ws = null
    reconnectTimer = setTimeout(connect, 3000)
  }

  ws.onerror = () => {
    ws?.close()
  }
}

function sendFilter() {
  if (!ws || ws.readyState !== WebSocket.OPEN) return
  const msg: any = {}
  if (filters.levels.length > 0) msg.level = filters.levels
  ws.send(JSON.stringify(msg))
}

function toggleConnection() {
  if (connected.value) {
    ws?.close()
    clearTimeout(reconnectTimer)
  } else {
    connect()
  }
}

watch(filters, () => {
  if (connected.value) sendFilter()
}, { deep: true })

onMounted(() => {
  connect()
})

onUnmounted(() => {
  ws?.close()
  clearTimeout(reconnectTimer)
})
</script>
