import { exec } from 'child_process'
import { promisify } from 'util'

const execAsync = promisify(exec)

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

async function getSystemResources(): Promise<SystemResources> {
  const isWindows = process.platform === 'win32'
  
  try {
    if (isWindows) {
      return await getWindowsResources()
    } else {
      return await getUnixResources()
    }
  } catch (error) {
    console.error('Error getting system resources:', error)
    throw new Error('Failed to get system resources')
  }
}

async function getUnixResources(): Promise<SystemResources> {
  const [cpuInfo, memoryInfo, diskInfo, uptimeInfo] = await Promise.all([
    getUnixCpuInfo(),
    getUnixMemoryInfo(),
    getUnixDiskInfo(),
    getUnixUptime()
  ])

  return {
    cpu: cpuInfo,
    memory: memoryInfo,
    disk: diskInfo,
    uptime: uptimeInfo,
    timestamp: Date.now()
  }
}

async function getUnixCpuInfo() {
  try {
    // Get CPU usage using top command
    const { stdout: topOutput } = await execAsync("top -l 1 -n 0 | grep 'CPU usage'")
    const cpuMatch = topOutput.match(/(\d+\.\d+)% user, (\d+\.\d+)% sys, (\d+\.\d+)% idle/)
    
    let cpuUsage = 0
    if (cpuMatch) {
      const user = parseFloat(cpuMatch[1])
      const sys = parseFloat(cpuMatch[2])
      cpuUsage = user + sys
    }

    // Get CPU cores
    const { stdout: coresOutput } = await execAsync("sysctl -n hw.ncpu")
    const cores = parseInt(coresOutput.trim())

    // Get load average
    const { stdout: loadOutput } = await execAsync("uptime")
    const loadMatch = loadOutput.match(/load averages?: ([\d.]+)[,\s]+([\d.]+)[,\s]+([\d.]+)/)
    const loadAverage = loadMatch ? [
      parseFloat(loadMatch[1]),
      parseFloat(loadMatch[2]),
      parseFloat(loadMatch[3])
    ] : [0, 0, 0]

    return {
      usage: Math.round(cpuUsage * 100) / 100,
      cores,
      loadAverage
    }
  } catch (error) {
    console.error('Error getting CPU info:', error)
    return { usage: 0, cores: 1, loadAverage: [0, 0, 0] }
  }
}

async function getUnixMemoryInfo() {
  try {
    // Try using system_profiler first (more reliable on macOS)
    try {
      const { stdout } = await execAsync("system_profiler SPHardwareDataType | grep 'Memory:'")
      const memoryMatch = stdout.match(/Memory:\s+(\d+)\s+GB/)
      if (memoryMatch) {
        const totalGB = parseInt(memoryMatch[1])
        const total = totalGB * 1024 * 1024 * 1024 // Convert to bytes
        
        // Get current memory usage
        const { stdout: vmStatOutput } = await execAsync("vm_stat")
        const lines = vmStatOutput.split('\n')
        
        let pageSize = 4096
        let free = 0
        let active = 0
        let inactive = 0
        let speculative = 0
        let wired = 0
        let compressed = 0

        for (const line of lines) {
          const match = line.match(/(\w+(?:\s+\w+)*):\s+(\d+)/)
          if (match) {
            const [, key, value] = match
            const pages = parseInt(value)
            switch (key.trim()) {
              case 'Pages free':
                free = pages
                break
              case 'Pages active':
                active = pages
                break
              case 'Pages inactive':
                inactive = pages
                break
              case 'Pages speculative':
                speculative = pages
                break
              case 'Pages wired down':
                wired = pages
                break
              case 'Pages stored in compressor':
                compressed = pages
                break
            }
          }
        }

        const used = (active + inactive + speculative + wired + compressed) * pageSize
        const freeMem = free * pageSize
        const usage = total > 0 ? (used / total) * 100 : 0

        return {
          total,
          used,
          free: freeMem,
          usage: Math.round(usage * 100) / 100
        }
      }
    } catch (error) {
      // system_profiler failed, trying alternative method
    }

    // Fallback: try using top command
    try {
      const { stdout } = await execAsync("top -l 1 -n 0 | grep 'PhysMem'")
      const physMemMatch = stdout.match(/PhysMem:\s+(\d+)M\s+used,\s+(\d+)M\s+wired,\s+(\d+)M\s+compressed\.\s+(\d+)M\s+unused/)
      if (physMemMatch) {
        const used = parseInt(physMemMatch[1]) * 1024 * 1024
        const wired = parseInt(physMemMatch[2]) * 1024 * 1024
        const compressed = parseInt(physMemMatch[3]) * 1024 * 1024
        const unused = parseInt(physMemMatch[4]) * 1024 * 1024
        
        const total = used + unused
        const usage = total > 0 ? (used / total) * 100 : 0

        return {
          total,
          used,
          free: unused,
          usage: Math.round(usage * 100) / 100
        }
      }
    } catch (error) {
      // top command failed, trying vm_stat
    }

    // Final fallback: use vm_stat with better parsing
    const { stdout } = await execAsync("vm_stat")
    const lines = stdout.split('\n')
    
    let pageSize = 4096
    let free = 0
    let active = 0
    let inactive = 0
    let speculative = 0
    let wired = 0
    let compressed = 0

    for (const line of lines) {
      const match = line.match(/(\w+(?:\s+\w+)*):\s+(\d+)/)
      if (match) {
        const [, key, value] = match
        const pages = parseInt(value)
        switch (key.trim()) {
          case 'Pages free':
            free = pages
            break
          case 'Pages active':
            active = pages
            break
          case 'Pages inactive':
            inactive = pages
            break
          case 'Pages speculative':
            speculative = pages
            break
          case 'Pages wired down':
            wired = pages
            break
          case 'Pages stored in compressor':
            compressed = pages
            break
        }
      }
    }

    const total = (free + active + inactive + speculative + wired + compressed) * pageSize
    const used = (active + inactive + speculative + wired + compressed) * pageSize
    const freeMem = free * pageSize
    const usage = total > 0 ? (used / total) * 100 : 0

    return {
      total,
      used,
      free: freeMem,
      usage: Math.round(usage * 100) / 100
    }
  } catch (error) {
    console.error('Error getting memory info:', error)
    return { total: 0, used: 0, free: 0, usage: 0 }
  }
}

async function getUnixDiskInfo() {
  try {
    const { stdout } = await execAsync("df -h /")
    const lines = stdout.split('\n')
    const dataLine = lines[1] // Skip header
    
    const parts = dataLine.split(/\s+/)
    const totalStr = parts[1]
    const usedStr = parts[2]
    const freeStr = parts[3]
    
    const parseSize = (sizeStr: string): number => {
      const match = sizeStr.match(/(\d+(?:\.\d+)?)([KMGTPE])/)
      if (!match) return 0
      
      const value = parseFloat(match[1])
      const unit = match[2]
      const multipliers: { [key: string]: number } = {
        'K': 1024,
        'M': 1024 * 1024,
        'G': 1024 * 1024 * 1024,
        'T': 1024 * 1024 * 1024 * 1024,
        'P': 1024 * 1024 * 1024 * 1024 * 1024,
        'E': 1024 * 1024 * 1024 * 1024 * 1024 * 1024
      }
      
      return value * (multipliers[unit] || 1)
    }
    
    const total = parseSize(totalStr)
    const used = parseSize(usedStr)
    const free = parseSize(freeStr)
    const usage = total > 0 ? (used / total) * 100 : 0
    
    return {
      total,
      used,
      free,
      usage: Math.round(usage * 100) / 100
    }
  } catch (error) {
    console.error('Error getting disk info:', error)
    return { total: 0, used: 0, free: 0, usage: 0 }
  }
}

async function getUnixUptime() {
  try {
    const { stdout } = await execAsync("uptime")
    const match = stdout.match(/up\s+([^,]+)/)
    if (match) {
      const uptimeStr = match[1]
      // Parse uptime string (e.g., "2 days, 3 hours, 45 minutes")
      const days = (uptimeStr.match(/(\d+)\s+day/)?.[1] || '0')
      const hours = (uptimeStr.match(/(\d+)\s+hour/)?.[1] || '0')
      const minutes = (uptimeStr.match(/(\d+)\s+minute/)?.[1] || '0')
      
      return parseInt(days) * 24 * 60 + parseInt(hours) * 60 + parseInt(minutes)
    }
    return 0
  } catch (error) {
    console.error('Error getting uptime:', error)
    return 0
  }
}

async function getWindowsResources(): Promise<SystemResources> {
  const [cpuInfo, memoryInfo, diskInfo, uptimeInfo] = await Promise.all([
    getWindowsCpuInfo(),
    getWindowsMemoryInfo(),
    getWindowsDiskInfo(),
    getWindowsUptime()
  ])

  return {
    cpu: cpuInfo,
    memory: memoryInfo,
    disk: diskInfo,
    uptime: uptimeInfo,
    timestamp: Date.now()
  }
}

async function getWindowsCpuInfo() {
  try {
    // Get CPU usage using wmic
    const { stdout: cpuOutput } = await execAsync('wmic cpu get loadpercentage /value')
    const cpuMatch = cpuOutput.match(/LoadPercentage=(\d+)/)
    const usage = cpuMatch ? parseFloat(cpuMatch[1]) : 0

    // Get CPU cores
    const { stdout: coresOutput } = await execAsync('wmic cpu get NumberOfCores /value')
    const coresMatch = coresOutput.match(/NumberOfCores=(\d+)/)
    const cores = coresMatch ? parseInt(coresMatch[1]) : 1

    // Windows doesn't have load average, so we'll use CPU usage as a proxy
    const loadAverage = [usage / 100, usage / 100, usage / 100]

    return {
      usage: Math.round(usage * 100) / 100,
      cores,
      loadAverage
    }
  } catch (error) {
    console.error('Error getting Windows CPU info:', error)
    return { usage: 0, cores: 1, loadAverage: [0, 0, 0] }
  }
}

async function getWindowsMemoryInfo() {
  try {
    // Get memory info using wmic
    const { stdout: memOutput } = await execAsync('wmic OS get TotalVisibleMemorySize,FreePhysicalMemory /value')
    
    const totalMatch = memOutput.match(/TotalVisibleMemorySize=(\d+)/)
    const freeMatch = memOutput.match(/FreePhysicalMemory=(\d+)/)
    
    const totalKB = totalMatch ? parseInt(totalMatch[1]) : 0
    const freeKB = freeMatch ? parseInt(freeMatch[1]) : 0
    
    const total = totalKB * 1024 // Convert to bytes
    const free = freeKB * 1024
    const used = total - free
    const usage = total > 0 ? (used / total) * 100 : 0

    return {
      total,
      used,
      free,
      usage: Math.round(usage * 100) / 100
    }
  } catch (error) {
    console.error('Error getting Windows memory info:', error)
    return { total: 0, used: 0, free: 0, usage: 0 }
  }
}

async function getWindowsDiskInfo() {
  try {
    // Get disk info using wmic
    const { stdout: diskOutput } = await execAsync('wmic logicaldisk where size>0 get size,freespace /value')
    
    let totalBytes = 0
    let freeBytes = 0
    
    const lines = diskOutput.split('\n')
    for (const line of lines) {
      if (line.includes('Size=')) {
        const sizeMatch = line.match(/Size=(\d+)/)
        if (sizeMatch) totalBytes += parseInt(sizeMatch[1])
      }
      if (line.includes('FreeSpace=')) {
        const freeMatch = line.match(/FreeSpace=(\d+)/)
        if (freeMatch) freeBytes += parseInt(freeMatch[1])
      }
    }
    
    const used = totalBytes - freeBytes
    const usage = totalBytes > 0 ? (used / totalBytes) * 100 : 0

    return {
      total: totalBytes,
      used,
      free: freeBytes,
      usage: Math.round(usage * 100) / 100
    }
  } catch (error) {
    console.error('Error getting Windows disk info:', error)
    return { total: 0, used: 0, free: 0, usage: 0 }
  }
}

async function getWindowsUptime() {
  try {
    // Get uptime using wmic
    const { stdout: uptimeOutput } = await execAsync('wmic os get lastbootuptime /value')
    const bootMatch = uptimeOutput.match(/LastBootUpTime=(\d{14})/)
    
    if (bootMatch) {
      const bootTime = bootMatch[1]
      // Parse YYYYMMDDHHMMSS format
      const year = parseInt(bootTime.substring(0, 4))
      const month = parseInt(bootTime.substring(4, 6)) - 1
      const day = parseInt(bootTime.substring(6, 8))
      const hour = parseInt(bootTime.substring(8, 10))
      const minute = parseInt(bootTime.substring(10, 12))
      const second = parseInt(bootTime.substring(12, 14))
      
      const bootDate = new Date(year, month, day, hour, minute, second)
      const now = new Date()
      const uptimeMs = now.getTime() - bootDate.getTime()
      return Math.floor(uptimeMs / (1000 * 60)) // Convert to minutes
    }
    
    return 0
  } catch (error) {
    console.error('Error getting Windows uptime:', error)
    return 0
  }
}

export default defineEventHandler(async (event) => {
  try {
    const resources = await getSystemResources()
    return resources
  } catch (error) {
    throw createError({
      statusCode: 500,
      statusMessage: 'Failed to get system resources'
    })
  }
})
