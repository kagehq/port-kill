import { spawn } from 'child_process'

export default defineEventHandler(async (event) => {
  try {
    // Get query parameters to determine which processes to kill
    const query = getQuery(event)
    const ports = query.ports || '3000-4000'
    const ignorePorts = query.ignorePorts || '5353,5000,7000'
    const ignoreProcesses = query.ignoreProcesses || 'Chrome,ControlCe,rapportd'
    
    // First, get all processes on the specified ports
    const processes = await getProcessesOnPorts(ports, ignorePorts, ignoreProcesses)
    
    if (processes.length === 0) {
      return {
        success: true,
        message: 'No processes found to kill',
        killedCount: 0,
        timestamp: new Date().toISOString()
      }
    }
    
    // Kill all processes
    const results = await Promise.allSettled(
      processes.map(process => killProcess(process.pid))
    )
    
    const successful = results.filter(result => result.status === 'fulfilled').length
    const failed = results.filter(result => result.status === 'rejected').length
    
    return {
      success: true,
      message: `Killed ${successful} processes${failed > 0 ? `, ${failed} failed` : ''}`,
      killedCount: successful,
      failedCount: failed,
      timestamp: new Date().toISOString()
    }
    
  } catch (error) {
    console.error('Error killing all processes:', error)
    
    throw createError({
      statusCode: 500,
      statusMessage: `Failed to kill all processes: ${error.message}`
    })
  }
})

async function getProcessesOnPorts(
  ports: string, 
  ignorePorts: string, 
  ignoreProcesses: string
): Promise<any[]> {
  return new Promise((resolve, reject) => {
    // Parse port range
    let portArgs: string[]
    if (ports.includes(',')) {
      // Specific ports
      portArgs = ports.split(',').map(p => `:${p.trim()}`)
    } else if (ports.includes('-')) {
      // Port range
      const [start, end] = ports.split('-').map(Number)
      portArgs = []
      for (let port = start; port <= end; port++) {
        portArgs.push(`:${port}`)
      }
    } else {
      // Single port
      portArgs = [`:${ports}`]
    }
    
    // Build lsof command with separate -i flags for each port
    const args = ['-sTCP:LISTEN', '-P', '-n']
    portArgs.forEach(port => {
      args.push('-i', port)
    })
    
    const lsof = spawn('lsof', args, {
      stdio: ['pipe', 'pipe', 'pipe']
    })
    
    let stdout = ''
    let stderr = ''
    
    lsof.stdout.on('data', (data) => {
      stdout += data.toString()
    })
    
    lsof.stderr.on('data', (data) => {
      stderr += data.toString()
    })
    
    lsof.on('close', (code) => {
      if (code !== 0) {
        reject(new Error(`lsof failed with code ${code}: ${stderr}`))
        return
      }
      
      try {
        const processes = parseLsofOutput(stdout, ignorePorts, ignoreProcesses)
        resolve(processes)
      } catch (error) {
        reject(error)
      }
    })
    
    lsof.on('error', (error) => {
      reject(error)
    })
  })
}

function parseLsofOutput(output: string, ignorePorts: string, ignoreProcesses: string): any[] {
  const processes: any[] = []
  const lines = output.split('\n')
  
  // Parse ignore lists
  const ignorePortsSet = new Set(ignorePorts.split(',').map(p => parseInt(p.trim())))
  const ignoreProcessesSet = new Set(ignoreProcesses.split(',').map(p => p.trim().toLowerCase()))
  
  for (let i = 1; i < lines.length; i++) { // Skip header
    const line = lines[i].trim()
    if (!line) continue
    
    const parts = line.split(/\s+/)
    if (parts.length < 9) continue
    
    const command = parts[0]
    const pid = parseInt(parts[1])
    const name = parts[8]
    
    // Extract port from name (e.g., "*:3000" or "127.0.0.1:3000")
    const portMatch = name.match(/:(\d+)$/)
    if (!portMatch) continue
    
    const port = parseInt(portMatch[1])
    
    // Check if should be ignored
    if (ignorePortsSet.has(port) || ignoreProcessesSet.has(command.toLowerCase())) {
      continue
    }
    
    processes.push({
      pid,
      port,
      command,
      name: command.split('/').pop() || command
    })
  }
  
  return processes
}

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