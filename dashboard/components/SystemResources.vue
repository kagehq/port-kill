<template>
  <div class="space-y-6 border-b border-gray-500/10 p-6">
    <!-- System Resources Header -->
    <div class="flex items-center justify-between">
      <h2 class="text-xs font-medium uppercase text-gray-500 flex items-center">
        System Resources
      </h2>
    </div>

    <!-- Resource Cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <!-- CPU Usage -->
      <div class="card p-6 group relative">
        <!-- Custom Tooltip -->
        <div class="absolute -top-2 left-1/2 transform -translate-x-1/2 -translate-y-full bg-[#0b0b10] border border-gray-500/10 text-white text-xs rounded-xl px-4 py-3 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-50 w-72">
          <div class="text-center">
            <div class="text-gray-400 text-left">Shows the percentage of CPU processing power currently being used. Higher values indicate more intensive processing tasks.</div>
          </div>
          <!-- Tooltip arrow -->
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
        </div>
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-blue-300/10 text-blue-300">
              <CpuChipIcon class="w-6 h-6" />
            </div>
            <div class="ml-3">
              <p class="text-sm font-medium text-gray-400">CPU</p>
              <p class="text-2xl font-bold text-white">{{ resources.cpu.usage }}%</p>
            </div>
          </div>
        </div>
        <div class="space-y-2">
          <div class="flex justify-between text-sm text-gray-400">
            <span>Cores: {{ resources.cpu.cores }}</span>
            <span>Load: {{ resources.cpu.loadAverage[0].toFixed(2) }}</span>
          </div>
          <div class="w-full bg-gray-500/10 rounded-full h-2">
            <div 
              class="bg-blue-300 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${Math.min(resources.cpu.usage, 100)}%` }"
            ></div>
          </div>
        </div>
      </div>

      <!-- Memory Usage -->
      <div class="card p-6 group relative">
        <!-- Custom Tooltip -->
        <div class="absolute -top-2 left-1/2 transform -translate-x-1/2 -translate-y-full bg-[#0b0b10] border border-gray-500/10 text-white text-xs rounded-xl px-4 py-3 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-50 w-72">
          <div class="text-center">
            <div class="text-gray-400 text-left">Shows the percentage of RAM currently being used. High usage may indicate memory leaks or insufficient RAM for current tasks.</div>
          </div>
          <!-- Tooltip arrow -->
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
        </div>
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-green-400/10 text-green-400">
              <CircleStackIcon class="w-6 h-6" />
            </div>
            <div class="ml-3">
              <p class="text-sm font-medium text-gray-400">Memory</p>
              <p class="text-2xl font-bold text-white">{{ resources.memory.usage }}%</p>
            </div>
          </div>
        </div>
        <div class="space-y-2">
          <div class="flex justify-between text-sm text-gray-400">
            <span>{{ formatBytes(resources.memory.used) }}</span>
            <span>{{ formatBytes(resources.memory.total) }}</span>
          </div>
          <div class="w-full bg-gray-500/10 rounded-full h-2">
            <div 
              class="bg-green-400 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${Math.min(resources.memory.usage, 100)}%` }"
            ></div>
          </div>
        </div>
      </div>

      <!-- Disk Usage -->
      <div class="card p-6 group relative">
        <!-- Custom Tooltip -->
        <div class="absolute -top-2 left-1/2 transform -translate-x-1/2 -translate-y-full bg-gray-500/10 border border-gray-500/10 text-white text-xs rounded-xl px-4 py-3 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-50 w-72">
          <div class="text-center">
            <div class="text-gray-400 text-left">Shows the percentage of storage space currently being used on your hard drive or SSD. High usage may require cleanup or additional storage.</div>
          </div>
          <!-- Tooltip arrow -->
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
        </div>
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-purple-400/10 text-purple-400">
              <ServerIcon class="w-6 h-6" />
            </div>
            <div class="ml-3">
              <p class="text-sm font-medium text-gray-400">Disk</p>
              <p class="text-2xl font-bold text-white">{{ resources.disk.usage }}%</p>
            </div>
          </div>
        </div>
        <div class="space-y-2">
          <div class="flex justify-between text-sm text-gray-400">
            <span>{{ formatBytes(resources.disk.used) }}</span>
            <span>{{ formatBytes(resources.disk.total) }}</span>
          </div>
          <div class="w-full bg-gray-500/10 rounded-full h-2">
            <div 
              class="bg-purple-400 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${Math.min(resources.disk.usage, 100)}%` }"
            ></div>
          </div>
        </div>
      </div>

      <!-- System Uptime -->
      <div class="card p-6 group relative">
        <!-- Custom Tooltip -->
        <div class="absolute -top-2 left-1/2 border border-gray-500/10 transform -translate-x-1/2 -translate-y-full bg-[#0b0b10] text-white text-xs rounded-lg px-3 py-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-50 w-72">
          <div class="text-center">
            <div class="text-gray-400 text-left">Shows how long the system has been running continuously without restart. Longer uptime indicates system stability and reliability.</div>
          </div>
          <!-- Tooltip arrow -->
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
        </div>
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-yellow-400/10 text-yellow-400">
              <ClockIcon class="w-6 h-6" />
            </div>
            <div class="ml-3">
              <p class="text-sm font-medium text-gray-400">Uptime</p>
              <p class="text-2xl font-bold text-white">{{ formatUptime(resources.uptime) }}</p>
            </div>
          </div>
        </div>
        <div class="text-sm text-gray-400">
          <div>Last updated: {{ formatTime(resources.timestamp) }}</div>
        </div>
      </div>
    </div>

    <!-- Load Average Cards -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <!-- 1 Minute Load Average -->
      <div class="card p-6 group relative">
        <!-- Custom Tooltip -->
        <div class="absolute -top-2 left-1/2 border border-gray-500/10 transform -translate-x-1/2 -translate-y-full bg-[#0b0b10] text-white text-xs rounded-lg px-3 py-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-50 w-72">
          <div class="text-center">
            <div class="text-gray-400 text-left">Shows the average system load over the past 1 minute. Values close to the number of CPU cores indicate optimal usage. Higher values suggest the system is overloaded.</div>
          </div>
          <!-- Tooltip arrow -->
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
        </div>
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-blue-400/10 text-blue-400">
              <BoltIcon class="w-6 h-6" />
            </div>
            <div class="ml-3">
              <p class="text-sm font-medium text-gray-400">1 Minute Load</p>
              <p class="text-2xl font-bold text-white">{{ resources.cpu.loadAverage[0].toFixed(2) }}</p>
            </div>
          </div>
        </div>
        <div class="text-sm text-gray-400">
          <div>Current load average</div>
        </div>
      </div>

      <!-- 5 Minute Load Average -->
      <div class="card p-6 group relative">
        <!-- Custom Tooltip -->
        <div class="absolute -top-2 left-1/2 border border-gray-500/10 transform -translate-x-1/2 -translate-y-full bg-[#0b0b10] text-white text-xs rounded-lg px-3 py-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-50 w-72">
          <div class="text-center">
            <div class="text-gray-400 text-left">Shows the average system load over the past 5 minutes. This provides a smoother view of system performance trends and helps identify sustained high load periods.</div>
          </div>
          <!-- Tooltip arrow -->
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
        </div>
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-green-400/10 text-green-400">
              <ChartBarSquareIcon class="w-6 h-6" />
            </div>
            <div class="ml-3">
              <p class="text-sm font-medium text-gray-400">5 Minute Load</p>
              <p class="text-2xl font-bold text-white">{{ resources.cpu.loadAverage[1].toFixed(2) }}</p>
            </div>
          </div>
        </div>
        <div class="text-sm text-gray-400">
          <div>Medium-term load average</div>
        </div>
      </div>

      <!-- 15 Minute Load Average -->
      <div class="card p-6 group relative">
        <!-- Custom Tooltip -->
        <div class="absolute -top-2 left-1/2 border border-gray-500/10 transform -translate-x-1/2 -translate-y-full bg-[#0b0b10] text-white text-xs rounded-lg px-3 py-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-50 w-72">
          <div class="text-center">
            <div class="text-gray-400 text-left">Shows the average system load over the past 15 minutes. This provides the most stable view of system performance and helps identify long-term trends and baseline system behavior.</div>
          </div>
          <!-- Tooltip arrow -->
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
        </div>
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center bg-purple-400/10 text-purple-400">
              <ChartPieIcon class="w-6 h-6" />
            </div>
            <div class="ml-3">
              <p class="text-sm font-medium text-gray-400">15 Minute Load</p>
              <p class="text-2xl font-bold text-white">{{ resources.cpu.loadAverage[2].toFixed(2) }}</p>
            </div>
          </div>
        </div>
        <div class="text-sm text-gray-400">
          <div>Long-term load average</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { 
  CpuChipIcon,
  CircleStackIcon,
  ClockIcon,
  BoltIcon,
  ChartBarSquareIcon,
  ChartPieIcon,
  ServerIcon
} from '@heroicons/vue/24/solid'
interface SystemResources {
  cpu: {
    usage: number
    cores: number
    loadAverage: number[]
  }
  memory: {
    total: number
    used: number
    free: number
    usage: number
  }
  disk: {
    total: number
    used: number
    free: number
    usage: number
  }
  uptime: number
  timestamp: number
}

// Reactive data - now handled by computed properties from useFetch

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatUptime = (minutes: number): string => {
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  const remainingMinutes = minutes % 60
  if (hours < 24) return `${hours}h ${remainingMinutes}m`
  const days = Math.floor(hours / 24)
  const remainingHours = hours % 24
  return `${days}d ${remainingHours}h`
}

const formatTime = (timestamp: number): string => {
  // Use a consistent format to avoid hydration mismatches
  const date = new Date(timestamp)
  return date.toISOString().substr(11, 8) // HH:MM:SS format
}

// Use useLazyFetch for initial data fetching
const { data: resources, error, pending: isLoading, refresh: refreshResources } = await useLazyFetch('/api/system/resources', {
  server: true, // Fetch on server side first
  default: () => ({
    cpu: { usage: 0, cores: 0, loadAverage: [0, 0, 0] },
    memory: { total: 0, used: 0, free: 0, usage: 0 },
    disk: { total: 0, used: 0, free: 0, usage: 0 },
    uptime: 0,
    timestamp: 0
  }),
  immediate: true // Ensure immediate fetch
})

// Computed property for connection status
const isConnected = computed(() => !error.value && resources.value && resources.value.cpu.usage > 0)

// Auto-refresh every 5 seconds using direct fetch to avoid scroll issues
let refreshInterval: NodeJS.Timeout | null = null

// Use direct fetch instead of refresh to avoid scroll jumping
const refreshData = async () => {
  try {
    const newData = await $fetch('/api/system/resources')
    if (newData) {
      resources.value = newData
    }
  } catch (err) {
    console.error('Error refreshing system resources:', err)
  }
}

onMounted(() => {
  refreshInterval = setInterval(refreshData, 5000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

// Expose refresh function to parent component
defineExpose({
  refreshData
})
</script>
