<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 overflow-y-auto">
    <!-- Backdrop -->
    <div class="fixed inset-0 bg-black bg-opacity-50 transition-opacity" @click="closeModal"></div>
    
    <!-- Modal -->
    <div class="flex min-h-full items-center justify-center p-4">
      <div class="relative bg-[#0b0b10] border border-gray-500/10 rounded-xl max-w-md w-full">
        <!-- Header -->
        <div class="flex items-center justify-between p-6 border-b border-gray-500/10">
          <div class="flex items-center space-x-3">
            <ExclamationTriangleIcon class="h-6 w-6 text-yellow-500" />
            <h3 class="text-lg font-semibold text-white">Confirm Process Termination</h3>
          </div>
          <button @click="closeModal" class="text-gray-500 hover:text-white transition-colors">
            <XMarkIcon class="h-5 w-5" />
          </button>
        </div>
        
        <!-- Content -->
        <div class="p-6">
          <div class="space-y-4">
            <!-- Process Info -->
            <div class="bg-gray-500/5 border border-gray-500/10 rounded-xl p-4">
              <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span class="text-gray-400">PID:</span>
                  <span class="text-white font-mono ml-2">{{ process.pid }}</span>
                </div>
                <div>
                  <span class="text-gray-400">Port:</span>
                  <span class="text-white font-mono ml-2">{{ process.port }}</span>
                </div>
                <div class="col-span-2">
                  <span class="text-gray-400">Name:</span>
                  <span class="text-white ml-2">{{ process.name }}</span>
                </div>
                <div class="col-span-2">
                  <span class="text-gray-400">Type:</span>
                  <span class="text-white ml-2">{{ processType }}</span>
                </div>
              </div>
            </div>
            
            
            <!-- Command Preview -->
            <div v-if="process.command_line" class="bg-gray-500/5 border border-gray-500/10 rounded-xl p-4">
              <div class="text-sm text-gray-400 mb-2">Command:</div>
              <div class="text-xs text-gray-300 font-mono break-all">{{ process.command_line }}</div>
            </div>

						<!-- Warning Message -->
            <div class="bg-yellow-500/5 border border-yellow-500/10 rounded-xl p-4">
              <div class="flex items-start space-x-3">
                <!-- <ExclamationTriangleIcon class="h-5 w-5 text-yellow-500 mt-0.5 flex-shrink-0" /> -->
                <div class="text-sm text-yellow-400">
                  <!-- <p class="font-medium mb-2">{{ warningTitle }}</p> -->
                  <p>{{ warningMessage }}</p>
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Actions -->
        <div class="flex items-center justify-end space-x-3 p-6 border-t border-gray-500/10">
          <button
            @click="closeModal"
            class="px-4 py-2 text-sm font-medium text-gray-300 bg-gray-500/5 border border-gray-500/10 rounded-xl transition-colors"
          >
            Cancel
          </button>
          <button
            @click="confirmKill"
            :disabled="isKilling"
            class="px-4 py-2 text-sm font-medium text-white bg-red-500 hover:bg-red-400 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors flex items-center space-x-2"
          >
            <!-- <ArrowPathIcon v-if="isKilling" class="h-4 w-4 animate-spin" /> -->
            <span>{{ isKilling ? 'Killing...' : 'Proceed' }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ExclamationTriangleIcon, XMarkIcon, ArrowPathIcon } from '@heroicons/vue/24/solid'

const props = defineProps({
  isOpen: {
    type: Boolean,
    default: false
  },
  process: {
    type: Object,
    default: () => ({})
  }
})

const emit = defineEmits(['close', 'confirm'])

const isKilling = ref(false)

const processType = computed(() => {
  if (props.process.container_id === 'docker-daemon') {
    return 'Docker Daemon Process'
  } else if (props.process.container_id && props.process.container_id !== 'host-process') {
    return 'Docker Container'
  } else {
    return 'Host Process'
  }
})

const warningTitle = computed(() => {
  if (props.process.container_id === 'docker-daemon') {
    return '⚠️ This will shut down Docker service'
  } else if (props.process.container_id && props.process.container_id !== 'host-process') {
    return '⚠️ This will stop the Docker container'
  } else {
    return '⚠️ This will terminate the process'
  }
})

const warningMessage = computed(() => {
  if (props.process.container_id === 'docker-daemon') {
    return 'Killing this process will shut down the entire Docker service. All running containers will be stopped and Docker will become unavailable until restarted. Note: Docker daemon processes may be system-protected and cannot be killed.'
  } else if (props.process.container_id && props.process.container_id !== 'host-process') {
    return 'This will gracefully stop the Docker container. The container can be restarted later if needed.'
  } else {
    return 'This will terminate the process immediately. Any unsaved work may be lost.'
  }
})

const closeModal = () => {
  if (!isKilling.value) {
    emit('close')
  }
}

const confirmKill = async () => {
  isKilling.value = true
  try {
    await emit('confirm', props.process)
  } finally {
    isKilling.value = false
  }
}
</script>
