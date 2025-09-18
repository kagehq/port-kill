<template>
  <div class="process-history p-6 pt-3">
    <!-- Header -->
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center space-x-2">
        <span v-if="history.length > 0" class="text-sm text-gray-500">You have {{ filteredHistory.length }} entries so far</span>
      </div>
    </div>

    <!-- Filters -->
    <div class="flex flex-wrap gap-4 mb-6">
      <div class="flex-1 min-w-48">
        <div class="relative">
          <select
            v-model="selectedGroup"
            @change="filterHistory"
            class="appearance-none w-full px-4 py-3 text-sm bg-transparent border border-gray-500/10 rounded-xl text-white focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 outline-none transition-all duration-200 hover:border-gray-500/30 cursor-pointer"
          >
            <option value="" class="bg-gray-800 text-white">All Groups</option>
            <option v-for="group in availableGroups" :key="group" :value="group" class="bg-gray-800 text-white">
              {{ group }}
            </option>
          </select>
          <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
            <ChevronDownIcon class="w-4 h-4 text-gray-400" />
          </div>
        </div>
      </div>
      <div class="flex-1 min-w-48">
        <div class="relative">
          <select
            v-model="selectedProject"
            @change="filterHistory"
            class="appearance-none w-full px-4 py-3 text-sm bg-transparent border border-gray-500/10 rounded-xl text-white focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 outline-none transition-all duration-200 hover:border-gray-500/30 cursor-pointer"
          >
            <option value="" class="bg-gray-800 text-white">All Projects</option>
            <option v-for="project in availableProjects" :key="project" :value="project" class="bg-gray-800 text-white">
              {{ project }}
            </option>
          </select>
          <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
            <ChevronDownIcon class="w-4 h-4 text-gray-400" />
          </div>
        </div>
      </div>
      <div class="flex-1 min-w-32">
        <div class="relative">
          <select
            v-model="timeFilter"
            @change="filterHistory"
            class="appearance-none w-full px-4 py-3 text-sm bg-transparent border border-gray-500/10 rounded-xl text-white focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 outline-none transition-all duration-200 hover:border-gray-500/30 cursor-pointer"
          >
            <option value="all" class="bg-gray-800 text-white">All Time</option>
            <option value="1h" class="bg-gray-800 text-white">Last Hour</option>
            <option value="24h" class="bg-gray-800 text-white">Last 24 Hours</option>
            <option value="7d" class="bg-gray-800 text-white">Last 7 Days</option>
          </select>
          <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
            <ChevronDownIcon class="w-4 h-4 text-gray-400" />
          </div>
        </div>
      </div>
    </div>

    <!-- History List -->
    <div class="space-y-3">
      <!-- Loading State -->
      <div v-if="isLoading" class="flex items-center justify-center py-8">
        <div class="flex items-center space-x-2 text-gray-400">
          <ArrowPathIcon class="w-5 h-5 animate-spin" />
          <span>Loading history...</span>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else-if="filteredHistory.length === 0" class="text-center py-12">
        <div class="flex flex-col items-center space-y-4">
          <ClockIcon class="w-12 h-12 text-gray-500" />
          <div>
            <h4 class="text-lg font-medium text-white mb-2">No History Found</h4>
            <p class="text-gray-400">
              {{ history.length === 0 ? 'No processes have been killed yet.' : 'No processes match your current filters.' }}
            </p>
          </div>
        </div>
      </div>

      <!-- History Entries -->
      <div v-else class="space-y-3">
        <div
          v-for="entry in filteredHistory"
          :key="`${entry.pid}-${entry.killed_at}`"
          class="bg-gray-500/5 border border-gray-500/10 rounded-xl p-4 hover:bg-gray-500/10 transition-colors duration-200"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center space-x-3 mb-2">
                <span class="font-medium text-white">{{ entry.process_name || entry.name }}</span>
                <span class="port-badge">{{ entry.port }}</span>
                <span v-if="entry.process_group" class="group-badge">{{ entry.process_group }}</span>
                <span v-if="entry.project_name" class="project-badge">{{ entry.project_name }}</span>
              </div>
              
              <div class="text-sm text-gray-400 space-y-1">
                <div class="flex items-center space-x-4">
                  <span>PID: {{ entry.pid }}</span>
                  <!-- <span>Killed by: {{ entry.killed_by }}</span>
                  <span class="text-gray-500">{{ formatTimeAgo(entry.killed_at) }}</span> -->
                </div>
                
                <!-- <div v-if="entry.command_line" class="font-mono text-xs bg-gray-500/10 rounded px-2 py-1 mt-2">
                  {{ entry.command_line }}
                </div> -->
                
                <div v-if="entry.working_directory" class="text-xs text-gray-500 mt-1">
                  📁 {{ entry.working_directory }}
                </div>
              </div>
            </div>
            
            <div class="flex items-center space-x-2 ml-4">
              <span class="text-xs text-gray-500">
                {{ formatDateTime(entry.killed_at) }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Pagination -->
    <div v-if="filteredHistory.length > 0 && history.length > 20" class="flex items-center justify-between mt-6 pt-4 border-t border-gray-500/10">
      <div class="text-sm text-gray-400">
        Showing {{ filteredHistory.length }} of {{ history.length }} entries
      </div>
      <div class="flex items-center space-x-2">
        <button
          @click="loadMore"
          :disabled="isLoading || filteredHistory.length >= history.length"
          class="px-3 py-2 text-sm rounded-xl border border-gray-500/10 text-white bg-gray-500/10 hover:bg-gray-500/15 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Load More
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { ArrowPathIcon, ClockIcon } from '@heroicons/vue/24/outline'
import { ChevronDownIcon } from '@heroicons/vue/24/solid'

// Props
const props = defineProps({
  autoRefresh: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['refresh', 'clear'])

// State
const history = ref([])
const isLoading = ref(false)
const selectedGroup = ref('')
const selectedProject = ref('')
const timeFilter = ref('all')
const limit = ref(50)

// Computed
const availableGroups = computed(() => {
  const groups = new Set()
  history.value.forEach(entry => {
    if (entry.process_group) {
      groups.add(entry.process_group)
    }
  })
  return Array.from(groups).sort()
})

const availableProjects = computed(() => {
  const projects = new Set()
  history.value.forEach(entry => {
    if (entry.project_name) {
      projects.add(entry.project_name)
    }
  })
  return Array.from(projects).sort()
})

const filteredHistory = computed(() => {
  let filtered = history.value

  // Filter by group
  if (selectedGroup.value) {
    filtered = filtered.filter(entry => entry.process_group === selectedGroup.value)
  }

  // Filter by project
  if (selectedProject.value) {
    filtered = filtered.filter(entry => entry.project_name === selectedProject.value)
  }

  // Filter by time
  if (timeFilter.value !== 'all') {
    const now = new Date()
    const cutoff = new Date()
    
    switch (timeFilter.value) {
      case '1h':
        cutoff.setHours(now.getHours() - 1)
        break
      case '24h':
        cutoff.setDate(now.getDate() - 1)
        break
      case '7d':
        cutoff.setDate(now.getDate() - 7)
        break
    }
    
    filtered = filtered.filter(entry => new Date(entry.killed_at) >= cutoff)
  }

  return filtered.slice(0, limit.value)
})

// Methods
const fetchHistory = async () => {
  try {
    isLoading.value = true
    const data = await $fetch('/api/history', {
      query: {
        limit: limit.value,
        group: selectedGroup.value || undefined,
        project: selectedProject.value || undefined
      }
    })
    
    if (data && data.success) {
      history.value = data.history || []
    }
  } catch (error) {
    console.error('Failed to fetch history:', error)
  } finally {
    isLoading.value = false
  }
}


const filterHistory = () => {
  // Filters are applied via computed property
}

const loadMore = () => {
  limit.value += 50
}

const formatTimeAgo = (dateString) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffMs = now - date
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  return `${diffDays}d ago`
}

const formatDateTime = (dateString) => {
  const date = new Date(dateString)
  return date.toLocaleString()
}

// Lifecycle
onMounted(() => {
  fetchHistory()
})

// Auto-refresh
let interval = null
if (props.autoRefresh) {
  onMounted(() => {
    interval = setInterval(fetchHistory, 30000) // Refresh every 30 seconds
  })
  onUnmounted(() => {
    if (interval) {
      clearInterval(interval)
    }
  })
}

// Expose methods to parent component
defineExpose({
  fetchHistory
})
</script>

<style scoped>
.port-badge {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-500/10 text-blue-400 border border-blue-500/20;
}

.group-badge {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-500/10 text-green-400 border border-green-500/20;
}

.project-badge {
  @apply inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-indigo-500/10 text-indigo-400 border border-indigo-500/20;
}
</style>
