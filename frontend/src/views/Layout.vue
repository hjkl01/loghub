<template>
  <el-container style="height: 100vh">
    <el-aside width="220px">
      <el-menu
        :default-active="route.path"
        router
        style="height: 100%"
        background-color="#304156"
        text-color="#bfcbd9"
        active-text-color="#409eff"
      >
        <div style="height: 60px; display: flex; align-items: center; justify-content: center; color: #fff; font-size: 20px; font-weight: bold">
          LogHub
        </div>
        <el-menu-item index="/logs">
          <el-icon><Search /></el-icon>
          <span>日志查询</span>
        </el-menu-item>
        <el-menu-item index="/logs/realtime">
          <el-icon><VideoPlay /></el-icon>
          <span>实时日志</span>
        </el-menu-item>
        <el-menu-item index="/swagger">
          <el-icon><Document /></el-icon>
          <span>API 文档</span>
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-container>
      <el-header style="display: flex; justify-content: flex-end; align-items: center; border-bottom: 1px solid #e6e6e6">
        <el-dropdown @command="handleCommand">
          <span style="cursor: pointer">
            {{ auth.user?.username }} ({{ auth.user?.role }})
            <el-icon><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="logout">退出登录</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </el-header>
      <el-main>
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

function handleCommand(command: string) {
  if (command === 'logout') {
    auth.logout()
    router.push('/login')
  }
}
</script>
