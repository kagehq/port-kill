<template>
  <div v-if="open" class="fixed inset-0 z-50 overflow-y-auto">
    <div class="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
      <!-- Background overlay -->
      <div 
        class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
        @click="$emit('update:open', false)"
      ></div>

      <!-- Modal panel -->
      <div class="inline-block align-bottom bg-white dark:bg-gray-800 rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
        <div class="bg-white dark:bg-gray-800 px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="sm:flex sm:items-start">
            <div class="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-blue-100 dark:bg-blue-900 sm:mx-0 sm:h-10 sm:w-10">
              <Cog6ToothIcon class="h-6 w-6 text-blue-600 dark:text-blue-400" />
            </div>
            <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left w-full">
              <h3 class="text-lg leading-6 font-medium text-gray-900 dark:text-white">
                Port Kill Settings
              </h3>
              <div class="mt-4 space-y-4">
                <!-- Port Range -->
                <div>
                  <label for="ports" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Port Range
                  </label>
                  <input
                    id="ports"
                    v-model="localConfig.ports"
                    type="text"
                    placeholder="3000-6000"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  />
                  <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                    Port range to monitor (e.g., 3000-6000 or specific ports: 3000,8000,8080)
                  </p>
                </div>

                <!-- Ignore Ports -->
                <div>
                  <label for="ignorePorts" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Ignore Ports
                  </label>
                  <input
                    id="ignorePorts"
                    v-model="localConfig.ignorePorts"
                    type="text"
                    placeholder="5353,5000,7000"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  />
                  <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                    Comma-separated list of ports to ignore (e.g., Chromecast, AirDrop)
                  </p>
                </div>

                <!-- Ignore Processes -->
                <div>
                  <label for="ignoreProcesses" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Ignore Processes
                  </label>
                  <input
                    id="ignoreProcesses"
                    v-model="localConfig.ignoreProcesses"
                    type="text"
                    placeholder="Chrome,ControlCe,rapportd"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  />
                  <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                    Comma-separated list of process names to ignore
                  </p>
                </div>

                <!-- Docker Support -->
                <div class="flex items-center">
                  <input
                    id="docker"
                    v-model="localConfig.docker"
                    type="checkbox"
                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 dark:border-gray-600 rounded"
                  />
                  <label for="docker" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                    Enable Docker container monitoring
                  </label>
                </div>

                <!-- Verbose Mode -->
                <div class="flex items-center">
                  <input
                    id="verbose"
                    v-model="localConfig.verbose"
                    type="checkbox"
                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 dark:border-gray-600 rounded"
                  />
                  <label for="verbose" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                    Enable verbose mode (show command line and working directory)
                  </label>
                </div>

                <!-- Refresh Interval -->
                <div>
                  <label for="refreshInterval" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    Refresh Interval (ms)
                  </label>
                  <select
                    id="refreshInterval"
                    v-model="localConfig.refreshInterval"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  >
                    <option :value="2000">2 seconds</option>
                    <option :value="5000">5 seconds</option>
                    <option :value="10000">10 seconds</option>
                    <option :value="30000">30 seconds</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <div class="bg-gray-50 dark:bg-gray-700 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
          <button
            @click="saveSettings"
            class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm"
          >
            Save Settings
          </button>
          <button
            @click="$emit('update:open', false)"
            class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 dark:border-gray-600 shadow-sm px-4 py-2 bg-white dark:bg-gray-800 text-base font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { Cog6ToothIcon } from '@heroicons/vue/24/outline'

const props = defineProps({
  open: {
    type: Boolean,
    default: false
  },
  config: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['update:open', 'save'])

const localConfig = ref({ ...props.config })

watch(() => props.config, (newConfig) => {
  localConfig.value = { ...newConfig }
}, { deep: true })

const saveSettings = () => {
  emit('save', { ...localConfig.value })
  emit('update:open', false)
}
</script>
