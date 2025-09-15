import { spawn } from 'child_process'

export default defineEventHandler(async (event) => {
  const pid = getRouterParam(event, 'pid')
  
  if (!pid) {
    throw createError({
      statusCode: 400,
      statusMessage: 'PID is required'
    })
  }
  
  const pidNum = parseInt(pid)
  if (isNaN(pidNum)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid PID format'
    })
  }
  
  try {
    // First, get the process details to determine how to kill it
    const processDetails = await getProcessDetails(pidNum)
    
    if (!processDetails) {
      throw createError({
        statusCode: 404,
        statusMessage: `Process ${pid} not found`
      })
    }
    
    // Handle different types of processes
    if (processDetails.containerId && processDetails.containerId !== 'host-process' && processDetails.containerId !== 'docker-daemon') {
      // This is a Docker container, use docker stop
      await killDockerContainer(processDetails.containerId)
    } else {
      // This is a host process or Docker daemon, use regular kill
      await killProcess(pidNum)
    }
    
    return {
      success: true,
      message: `Process ${pid} killed successfully`,
      timestamp: new Date().toISOString()
    }
    
  } catch (error) {
    console.error(`Error killing process ${pid}:`, error)
    
    throw createError({
      statusCode: 500,
      statusMessage: `Failed to kill process ${pid}: ${error.message}`
    })
  }
})

async function killProcess(pid: number): Promise<void> {
  return new Promise((resolve, reject) => {
    // First try SIGTERM (graceful termination)
    const kill = spawn('kill', ['-TERM', pid.toString()], {
      stdio: ['pipe', 'pipe', 'pipe']
    })
    
    kill.on('close', (code) => {
      // Wait a bit for graceful termination
      setTimeout(() => {
        // Check if process is still running
        checkProcessExists(pid).then(exists => {
          if (exists) {
            // Process still running, send SIGKILL
            const forceKill = spawn('kill', ['-KILL', pid.toString()], {
              stdio: ['pipe', 'pipe', 'pipe']
            })
            
            forceKill.on('close', (forceCode) => {
              // Check again if process still exists after SIGKILL
              checkProcessExists(pid).then(stillExists => {
                if (!stillExists) {
                  resolve() // Process was killed successfully
                } else {
                  reject(new Error(`Failed to force kill process ${pid}`))
                }
              }).catch(reject)
            })
            
            forceKill.on('error', (error) => {
              reject(error)
            })
          } else {
            resolve() // Process was killed by SIGTERM
          }
        }).catch(reject)
      }, 500)
    })
    
    kill.on('error', (error) => {
      reject(error)
    })
  })
}

async function getProcessDetails(pid: number): Promise<any> {
  return new Promise((resolve) => {
    const ps = spawn('ps', ['-p', pid.toString(), '-o', 'pid,command'], {
      stdio: ['pipe', 'pipe', 'pipe']
    })
    
    let output = ''
    ps.stdout.on('data', (data) => {
      output += data.toString()
    })
    
    ps.on('close', (code) => {
      if (code === 0 && output.trim()) {
        const lines = output.trim().split('\n')
        if (lines.length > 1) {
          const parts = lines[1].trim().split(/\s+/)
          if (parts.length >= 2) {
            const command = parts.slice(1).join(' ')
            const containerId = command.includes('com.docke') ? 'docker-daemon' : null
            const containerName = command.includes('com.docke') ? 'Docker Daemon' : null
            
            resolve({
              pid: parseInt(parts[0]),
              command,
              containerId,
              containerName
            })
            return
          }
        }
      }
      resolve(null)
    })
    
    ps.on('error', () => {
      resolve(null)
    })
  })
}

async function killDockerContainer(containerId: string): Promise<void> {
  return new Promise((resolve, reject) => {
    // First try docker stop (graceful)
    const stop = spawn('docker', ['stop', containerId], {
      stdio: ['pipe', 'pipe', 'pipe']
    })
    
    stop.on('close', (code) => {
      if (code === 0) {
        resolve()
      } else {
        // If stop failed, try docker kill (force)
        const kill = spawn('docker', ['kill', containerId], {
          stdio: ['pipe', 'pipe', 'pipe']
        })
        
        kill.on('close', (killCode) => {
          if (killCode === 0) {
            resolve()
          } else {
            reject(new Error(`Failed to stop Docker container ${containerId}`))
          }
        })
        
        kill.on('error', (error) => {
          reject(error)
        })
      }
    })
    
    stop.on('error', (error) => {
      reject(error)
    })
  })
}

async function checkProcessExists(pid: number): Promise<boolean> {
  return new Promise((resolve) => {
    const ps = spawn('ps', ['-p', pid.toString()], {
      stdio: ['pipe', 'pipe', 'pipe']
    })
    
    ps.on('close', (code) => {
      resolve(code === 0)
    })
    
    ps.on('error', () => {
      resolve(false)
    })
  })
}