<template>
  <div class="h-screen bg-black flex">
    <!-- Left Sidebar -->
    <Sidebar :is-connected="isConnected" />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col mr-2 my-2 rounded-xl bg-gray-500/10 border border-gray-500/10 overflow-hidden">
      <!-- Top Header -->
      <header class="border-b border-gray-500/10">
        <div class="px-6 py-3">
          <div class="flex justify-between items-center">
            <div class="flex items-center space-x-2">
              <h2 class="text-base font-medium text-white">Process History</h2>
              <p class="text-sm text-gray-500">track killed processes and system activity</p>
            </div>
            
            <div class="flex items-center space-x-4">
              <!-- Auto-refresh Toggle Button -->
              <button
                @click="toggleAutoRefresh"
                :class="[
                  'flex items-center space-x-2 px-3 py-2 text-sm rounded-xl transition-colors duration-200',
                  isAutoRefreshEnabled 
                    ? 'bg-transparent text-gray-400 border border-gray-500/10 hover:bg-gray-500/15' 
                    : 'bg-orange-400/10 text-orange-400 hover:bg-orange-400/15'
                ]"
                :title="isAutoRefreshEnabled ? 'Pause auto-refresh' : 'Resume auto-refresh'"
              >
                <PlayIcon v-if="!isAutoRefreshEnabled" class="w-4 h-4" />
                <PauseIcon v-else class="w-4 h-4" />
                <span>{{ isAutoRefreshEnabled ? 'Pause' : 'Resume' }}</span>
              </button>
              
              <!-- Refresh Button -->
              <button
                @click="refreshData"
                :disabled="isLoading"
                class="border border-gray-500/10 text-sm rounded-xl px-4 py-2 text-white bg-gray-500/10 hover:bg-gray-500/15 disabled:opacity-50 disabled:cursor-not-allowed flex items-center space-x-2"
              >
                <ArrowPathIcon 
                  :class="['w-4 h-4', isLoading ? 'animate-spin' : '']" 
                />
                <span>{{ isLoading ? 'Refreshing...' : 'Refresh' }}</span>
              </button>

              <span class="text-sm text-gray-500/10">|</span>

              <!-- Clear Button -->
              <button
                @click="clearHistory"
                :disabled="isLoading"
                class="flex items-center space-x-2 px-3 py-2 text-sm rounded-xl border border-gray-500/10 text-white bg-gray-500/10 hover:bg-gray-500/15 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <TrashIcon class="w-4 h-4" />
                <span>Clear</span>
              </button>
            </div>
          </div>
        </div>
      </header>

      <!-- Main Content -->
      <main class="flex-1 overflow-y-auto">
        <div class="p-6 pt-3">
          <ProcessHistory 
            ref="processHistoryRef"
            :auto-refresh="isAutoRefreshEnabled"
            @refresh="refreshData"
            @clear="refreshData"
          />
        </div>
      </main>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { ArrowPathIcon, PlayIcon, PauseIcon, TrashIcon } from '@heroicons/vue/24/solid'
import Sidebar from '@/components/Sidebar.vue'
import ProcessHistory from '@/components/ProcessHistory.vue'

// State
const isConnected = ref(true)
const isLoading = ref(false)
const isAutoRefreshEnabled = ref(true)
const processHistoryRef = ref(null)

// Auto-refresh interval
let refreshInterval = null

// Methods
const toggleAutoRefresh = () => {
  isAutoRefreshEnabled.value = !isAutoRefreshEnabled.value
  
  if (isAutoRefreshEnabled.value) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
}

const startAutoRefresh = () => {
  if (refreshInterval) return
  
  refreshInterval = setInterval(() => {
    refreshData(false)
  }, 30000) // Refresh every 30 seconds
}

const stopAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  }
}

const refreshData = async (showLoading = true) => {
  try {
    if (showLoading) {
      isLoading.value = true
    }
    
    // Call the ProcessHistory component's fetchHistory method
    if (processHistoryRef.value) {
      await processHistoryRef.value.fetchHistory()
    }
    
    isConnected.value = true
  } catch (error) {
    console.error('Failed to refresh data:', error)
    isConnected.value = false
  } finally {
    if (showLoading) {
      isLoading.value = false
    }
  }
}

const clearHistory = async () => {
  if (confirm('Are you sure you want to clear all process history? This action cannot be undone.')) {
    try {
      isLoading.value = true
      await $fetch('/api/history/clear', { method: 'POST' })
      
      // Refresh the history after clearing
      if (processHistoryRef.value) {
        await processHistoryRef.value.fetchHistory()
      }
    } catch (error) {
      console.error('Failed to clear history:', error)
      alert('Failed to clear history. Please try again.')
    } finally {
      isLoading.value = false
    }
  }
}

// Lifecycle
onMounted(() => {
  if (isAutoRefreshEnabled.value) {
    startAutoRefresh()
  }
})

onUnmounted(() => {
  stopAutoRefresh()
})

// Meta
useHead({
  title: 'Process History',
  meta: [
    { name: 'description', content: 'Track and view process kill history' }
  ]
})
</script>
