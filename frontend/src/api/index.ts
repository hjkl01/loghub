import axios from 'axios'

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
})

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token')
      localStorage.removeItem('user')
      window.location.href = '/login'
    }
    return Promise.reject(error)
  },
)

export interface ApiResponse<T = any> {
  code: number
  message: string
  data: T
}

export async function login(username: string, password: string) {
  const res = await api.post<ApiResponse>('/auth/login', { username, password })
  return res.data
}

export async function getMe() {
  const res = await api.get<ApiResponse>('/auth/me')
  return res.data
}

export interface LogItem {
  time: string
  ingest_time: string
  level: string
  system: string
  service: string
  message: string
  trace_id?: string
  request_id?: string
  file_name?: string
  function_name?: string
  line_number?: number
}

export interface LogQueryResult {
  total: number
  data: LogItem[]
}

export interface LogQueryParams {
  system?: string
  service?: string
  level?: string
  keyword?: string
  file_name?: string
  function_name?: string
  start_time?: string
  end_time?: string
  page?: number
  size?: number
}

export async function queryLogs(params: LogQueryParams) {
  const res = await api.get<ApiResponse<LogQueryResult>>('/logs', { params })
  return res.data
}

export default api
