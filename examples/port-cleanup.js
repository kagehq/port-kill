// Port Cleanup Script
// Automatically cleans up development ports

log("Starting port cleanup script")

// Clean up common development ports
killPort(3000)
killPort(3001)
killPort(5000)
killPort(8000)
killPort(8080)
killPort(9000)

log("Port cleanup completed")

// Monitor for new processes
onPort(3000, callback)
onPort(8080, callback)

log("Monitoring active - press Ctrl+C to stop")
