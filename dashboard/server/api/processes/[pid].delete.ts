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
    // Use system kill command
    const result = await killProcess(pidNum)
    
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