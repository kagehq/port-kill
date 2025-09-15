<template>
  <div class="p-0 border-r border-gray-500/10">
    <div class="flex items-center">
      <div class="flex-shrink-0">
        <div :class="[
          'w-10 h-10 rounded-lg flex items-center justify-center',
          colorClasses.bg,
          colorClasses.text
        ]">
          <component :is="icon" class="w-6 h-6" />
        </div>
      </div>
      <div class="ml-4 flex-1">
        <p class="text-sm font-medium text-gray-400">
          {{ title }}
        </p>
        <div class="flex items-baseline">
          <p class="text-2xl font-bold text-white">
            {{ value }}
          </p>
          <p v-if="change !== 0" :class="[
            'ml-2 text-sm font-medium',
            change > 0 ? 'text-green-400' : 'text-red-500'
          ]">
            {{ change > 0 ? '+' : '' }}{{ change }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  title: {
    type: String,
    required: true
  },
  value: {
    type: [String, Number],
    required: true
  },
  change: {
    type: Number,
    default: 0
  },
  icon: {
    type: [String, Object, Function],
    required: true
  },
  color: {
    type: String,
    default: 'blue',
    validator: (value) => ['blue', 'green', 'purple', 'yellow', 'orange', 'red'].includes(value)
  }
})

const colorClasses = computed(() => {
  const colors = {
    blue: {
      bg: 'bg-blue-500/10',
      text: 'text-blue-400'
    },
    green: {
      bg: 'bg-green-500/10',
      text: 'text-green-400'
    },
    purple: {
      bg: 'bg-purple-500/10',
      text: 'text-purple-400'
    },
    yellow: {
      bg: 'bg-yellow-500/10',
      text: 'text-yellow-400'
    },
    orange: {
      bg: 'bg-orange-500/10',
      text: 'text-orange-400'
    },
    red: {
      bg: 'bg-red-500/10',
      text: 'text-red-400'
    }
  }
  
  return colors[props.color] || colors.blue
})
</script>
