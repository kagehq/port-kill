<template>
  <div>
    <!-- Confirmation Modal -->
    <KillConfirmModal
      :is-open="showKillModal"
      :process="selectedProcess"
      @close="closeKillModal"
      @confirm="handleKillConfirm"
    />
    
    <!-- Process Table -->
    <div class="overflow-auto border-t border-gray-500/10">
    <table class="process-table">
      <thead class="bg-transparent border-b border-gray-500/10">
        <tr>
          <th class="w-16">Status</th>
          <th class="w-20">Port</th>
          <th class="w-24">PID</th>
          <th>Process Name</th>
          <th>Command</th>
          <th>Group</th>
          <th>Project</th>
          <th class="w-20">CPU</th>
          <th class="w-20">Memory</th>
          <th class="">Container</th>
          <th class="">Directory</th>
          <th class="w-24"></th>
        </tr>
      </thead>
      <tbody class="bg-transparent divide-y divide-gray-500/10">
        <tr v-if="isLoading" class="animate-pulse">
          <td colspan="12" class="px-6 py-8 text-center text-gray-400">
            <div class="flex items-center justify-center space-x-2">
              <ArrowPathIcon class="w-5 h-5 animate-spin" />
              <span>Loading processes...</span>
            </div>
          </td>
        </tr>
        <tr v-else-if="processes.length === 0">
          <td colspan="12" class="px-6 py-8 text-center text-gray-400">
            <div class="flex flex-col items-center space-y-2 border border-gray-500/10 rounded-xl max-w-md mx-auto p-10">
              <ServerIcon class="w-10 h-10 text-gray-500" />
              <p class="text-lg font-medium">No processes found</p>
              <p class="text-sm text-gray-400">Start a development server to see processes here</p>
            </div>
          </td>
        </tr>
        <tr v-else v-for="process in processes" :key="`${process.pid}-${process.port}`" 
            :class="[
              'hover:bg-gray-500/10',
              hasPortConflict(process) ? 'bg-transparent' : ''
            ]">
          <!-- Status -->
          <td class="px-6 py-4">
            <div class="flex items-center">
              <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse-slow"></div>
            </div>
          </td>
          
          <!-- Port -->
          <td class="px-6 py-4">
            <span class="port-badge">
              {{ process.port }}
            </span>
          </td>
          
          <!-- PID -->
          <td class="px-6 py-4 font-mono text-sm text-gray-400">
            {{ process.pid }}
          </td>
          
          <!-- Process Name -->
          <td class="px-6 py-4">
            <div class="flex items-center space-x-2">
              <span class="font-medium text-white">
                {{ process.name }}
              </span>
              <span v-if="process.container_id === 'docker-daemon'" class="docker-container">
                Docker
              </span>
              <span v-else-if="process.container_id === 'host-process'" class="host-container">
                Host
              </span>
              <span v-else-if="process.container_id" class="container-badge">
                Container
              </span>
              <div v-if="hasPortConflict(process)" class="flex items-center space-x-1 bg-orange-500/10 rounded-full px-2 py-0.5" title="Port conflict detected - multiple containers using same port">
                <ExclamationTriangleIcon class="w-3 h-3 text-orange-400" />
                <span class="text-xs text-orange-400 font-medium">Conflict</span>
              </div>
            </div>
          </td>
          
          <!-- Command -->
          <td class="px-6 py-4">
            <div class="max-w-xs truncate group relative" :title="process.command_line || process.command">
              <span class="text-sm text-gray-400 font-mono">
                {{ getShortCommand(process) }}
              </span>
              <!-- Full command tooltip on hover -->
              <div class="absolute -top-2 left-1/2 transform -translate-x-1/2 -translate-y-full bg-[#0b0b10] border border-gray-500/10 text-white text-xs rounded-lg px-3 py-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-[100] w-72">
                <div class="text-center">
                  <div class="text-gray-400 text-left">
                    {{ process.command_line || process.command }}
                  </div>
                </div>
                <!-- Tooltip arrow -->
                <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
              </div>
            </div>
          </td>
          
          <!-- Group -->
          <td class="px-6 py-4">
            <span v-if="process.process_group" class="group-badge">
              {{ process.process_group }}
            </span>
            <span v-else class="text-sm text-gray-500">-</span>
          </td>
          
          <!-- Project -->
          <td class="px-6 py-4">
            <span v-if="process.project_name" class="project-badge">
              {{ process.project_name }}
            </span>
            <span v-else class="text-sm text-gray-500">-</span>
          </td>
          
          <!-- CPU Usage -->
          <td class="px-6 py-4">
            <div v-if="process.cpu_usage !== null && process.cpu_usage !== undefined" class="flex items-center space-x-1">
              <span class="text-sm font-mono" :class="getCpuUsageClass(process.cpu_usage)">
                {{ process.cpu_usage.toFixed(1) }}%
              </span>
              <span v-if="process.cpu_usage > 50" class="text-red-400">🔥</span>
              <span v-else-if="process.cpu_usage > 20" class="text-yellow-400">⚡</span>
              <span v-else class="text-green-400">✅</span>
            </div>
            <span v-else class="text-sm text-gray-500">-</span>
          </td>
          
          <!-- Memory Usage -->
          <td class="px-6 py-4">
            <div v-if="process.memory_usage !== null && process.memory_usage !== undefined" class="flex items-center space-x-1">
              <span class="text-sm font-mono" :class="getMemoryUsageClass(process.memory_percentage)">
                {{ formatMemory(process.memory_usage) }}
              </span>
              <span v-if="process.memory_percentage > 5" class="text-red-400">🔥</span>
              <span v-else-if="process.memory_percentage > 2" class="text-yellow-400">⚡</span>
              <span v-else class="text-green-400">✅</span>
            </div>
            <span v-else class="text-sm text-gray-500">-</span>
          </td>
          
          <!-- Container -->
          <td class="px-6 py-4">
            <div v-if="process.container_name" class="flex items-center space-x-1">
              <CubeIcon class="w-4 h-4 text-purple-400" />
              <span class="text-sm text-gray-400 truncate max-w-24" :title="process.container_name">
                {{ process.container_name }}
              </span>
            </div>
            <span v-else class="text-sm text-gray-500">-</span>
          </td>
          
          <!-- Directory -->
          <td class="px-6 py-4">
            <div v-if="process.working_directory" class="max-w-32 truncate group relative" :title="process.working_directory">
              <span class="text-sm text-gray-400 font-mono">
                {{ getShortDirectory(process) }}
              </span>
              <!-- Full directory tooltip on hover -->
              <div class="absolute -top-2 left-1/2 transform -translate-x-1/2 -translate-y-full bg-[#0b0b10] border border-gray-500/10 text-white text-xs rounded-lg px-3 py-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none z-[100] w-72">
                <div class="text-center">
                  <div class="text-gray-400 text-left font-mono">
                    {{ process.working_directory }}
                  </div>
                </div>
                <!-- Tooltip arrow -->
                <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-500/10"></div>
              </div>
            </div>
            <span v-else class="text-sm text-gray-500">-</span>
          </td>
          
          <!-- Actions -->
          <td class="px-6 py-4 text-right">
            <div class="flex items-center justify-end space-x-2">
              <button
                @click="openKillModal(process)"
                class="text-gray-500 hover:text-red-500 transition-colors duration-200"
                :title="`Kill process ${process.pid}`"
              >
                <ArchiveBoxXMarkIcon class="w-4 h-4" />
              </button>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
    </div>
  </div>
</template>


<script setup>
import { computed, ref } from 'vue'
import { 
  ServerIcon, 
  CubeIcon,
  ExclamationTriangleIcon,
  ArchiveBoxXMarkIcon,
  ArrowPathIcon
} from '@heroicons/vue/24/solid'
import KillConfirmModal from './KillConfirmModal.vue'

const props = defineProps({
  processes: {
    type: Array,
    default: () => []
  },
  isLoading: {
    type: Boolean,
    default: false
  },
  hasPortConflict: {
    type: Function,
    default: () => false
  }
})

const emit = defineEmits(['kill-process'])

// Modal state
const showKillModal = ref(false)
const selectedProcess = ref({})

// Use the conflict detection function passed from parent
const hasPortConflict = props.hasPortConflict

// Modal methods
const openKillModal = (process) => {
  selectedProcess.value = process
  showKillModal.value = true
}

const closeKillModal = () => {
  showKillModal.value = false
  selectedProcess.value = {}
}

const handleKillConfirm = async (process) => {
  emit('kill-process', process)
  closeKillModal()
}

// Function to get a shortened version of the command for display
const getShortCommand = (process) => {
  const command = process.command_line || process.command
  
  if (!command) return '-'
  
  // For Node.js processes, extract the main script or command
  if (command.includes('node')) {
    // For Nuxt dev server
    if (command.includes('nuxt dev')) {
      return 'nuxt dev'
    }
    
    // For one-liner Node.js scripts, extract the key part
    if (command.includes('const http = require')) {
      // Extract the port number from the one-liner
      const portMatch = command.match(/server\.listen\((\d+)/)
      if (portMatch) {
        return `Test Server (port ${portMatch[1]})`
      }
      return 'Node.js Server'
    }
    
    // For other Node.js commands, extract the main script
    const match = command.match(/node\s+(.+?)(?:\s|$)/)
    if (match) {
      const script = match[1]
      // If it's a file path, show just the filename
      if (script.includes('/')) {
        return script.split('/').pop()
      }
      return script
    }
  }
  
  // For Docker processes
  if (command.includes('com.docke')) {
    return 'Docker'
  }
  
  // For other processes, show first 30 characters
  return command.length > 30 ? command.substring(0, 30) + '...' : command
}

// Function to get a shortened version of the directory for display
const getShortDirectory = (process) => {
  const directory = process.working_directory
  
  if (!directory) return '-'
  
  // For Docker processes, show a simple label
  if (directory.includes('com.docker.docker')) {
    return 'Docker'
  }
  
  // For regular directories, show the last folder name
  const parts = directory.split('/')
  if (parts.length > 1) {
    // Get the last non-empty part
    for (let i = parts.length - 1; i >= 0; i--) {
      if (parts[i] && parts[i] !== '') {
        return parts[i]
      }
    }
  }
  
  // If it's just a single folder or root, show it as is
  return directory.length > 20 ? directory.substring(0, 20) + '...' : directory
}

// Function to get CPU usage class for styling
const getCpuUsageClass = (cpuUsage) => {
  if (cpuUsage > 50) return 'text-red-400'
  if (cpuUsage > 20) return 'text-yellow-400'
  return 'text-green-400'
}

// Function to get memory usage class for styling
const getMemoryUsageClass = (memoryPercentage) => {
  if (memoryPercentage > 5) return 'text-red-400'
  if (memoryPercentage > 2) return 'text-yellow-400'
  return 'text-green-400'
}

// Function to format memory usage
const formatMemory = (bytes) => {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}
</script>

<style scoped>
.port-badge {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-500/10 text-blue-400 border border-blue-500/20;
}

.docker-container {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-purple-500/10 text-purple-400 border border-purple-500/20;
}

.host-container {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-500/10 text-gray-400 border border-gray-500/20;
}

.container-badge {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-orange-500/10 text-orange-400 border border-orange-500/20;
}

.group-badge {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-500/10 text-green-400 border border-green-500/20;
}

.project-badge {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-indigo-500/10 text-indigo-400 border border-indigo-500/20;
}

.process-table {
  @apply w-full;
}

.process-table th {
  @apply px-6 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider;
}

.process-table td {
  @apply whitespace-nowrap;
}

.animate-pulse-slow {
  animation: pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>
