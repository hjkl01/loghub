<template>
  <div>
    <h2 style="margin-bottom: 16px">日志查询</h2>

    <el-card style="margin-bottom: 16px">
      <el-form :inline="true" :model="filters">
        <el-form-item label="系统">
          <el-input v-model="filters.system" placeholder="系统名称" clearable style="width: 150px" />
        </el-form-item>
        <el-form-item label="服务">
          <el-input v-model="filters.service" placeholder="服务名称" clearable style="width: 150px" />
        </el-form-item>
        <el-form-item label="级别">
          <el-select v-model="filters.level" placeholder="全部级别" clearable style="width: 120px">
            <el-option label="DEBUG" value="DEBUG" />
            <el-option label="INFO" value="INFO" />
            <el-option label="WARN" value="WARN" />
            <el-option label="ERROR" value="ERROR" />
          </el-select>
        </el-form-item>
        <el-form-item label="关键词">
          <el-input v-model="filters.keyword" placeholder="搜索消息" clearable style="width: 200px" />
        </el-form-item>
        <el-form-item label="开始时间">
          <el-date-picker v-model="filters.start_time" type="datetime" placeholder="开始时间" style="width: 180px" />
        </el-form-item>
        <el-form-item label="结束时间">
          <el-date-picker v-model="filters.end_time" type="datetime" placeholder="结束时间" style="width: 180px" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">查询</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card>
      <div style="margin-bottom: 12px; color: #909399; font-size: 13px">
        共 {{ total }} 条记录
      </div>
      <el-table :data="logs" border stripe style="width: 100%" @row-click="showDetail">
        <el-table-column prop="time" label="时间" width="180">
          <template #default="{ row }">
            {{ formatTime(row.time) }}
          </template>
        </el-table-column>
        <el-table-column prop="level" label="级别" width="80">
          <template #default="{ row }">
            <el-tag :type="levelTag(row.level)" size="small">{{ row.level }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="system" label="系统" width="120" />
        <el-table-column prop="service" label="服务" width="120" />
        <el-table-column prop="message" label="消息" show-overflow-tooltip />
        <el-table-column prop="trace_id" label="Trace ID" width="160" />
        <el-table-column prop="request_id" label="Request ID" width="160" />
      </el-table>
      <div style="margin-top: 16px; display: flex; justify-content: center">
        <el-pagination
          v-model:current-page="page"
          :page-size="pageSize"
          :total="total"
          layout="prev, pager, next"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { queryLogs } from '../api'

const logs = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)

const filters = reactive({
  system: undefined as string | undefined,
  service: undefined as string | undefined,
  level: undefined as string | undefined,
  keyword: undefined as string | undefined,
  start_time: undefined as Date | undefined,
  end_time: undefined as Date | undefined,
})

function formatTime(t: string) {
  if (!t) return ''
  return t.replace('T', ' ').substring(0, 19)
}

function levelTag(level: string) {
  if (level === 'ERROR') return 'danger'
  if (level === 'WARN') return 'warning'
  if (level === 'DEBUG') return 'info'
  return 'success'
}

async function fetchLogs() {
  const params: any = { page: page.value, size: pageSize.value }
  if (filters.system) params.system = filters.system
  if (filters.service) params.service = filters.service
  if (filters.level) params.level = filters.level
  if (filters.keyword) params.keyword = filters.keyword
  if (filters.start_time) params.start_time = filters.start_time.toISOString()
  if (filters.end_time) params.end_time = filters.end_time.toISOString()

  const res = await queryLogs(params)
  logs.value = res.data?.data || []
  total.value = res.data?.total || 0
}

function handleSearch() {
  page.value = 1
  fetchLogs()
}

function handleReset() {
  filters.system = undefined
  filters.service = undefined
  filters.level = undefined
  filters.keyword = undefined
  filters.start_time = undefined
  filters.end_time = undefined
  page.value = 1
  fetchLogs()
}

function handlePageChange(p: number) {
  page.value = p
  fetchLogs()
}

function showDetail(row: any) {
  // Future: open detail dialog
}

onMounted(() => {
  fetchLogs()
})
</script>
