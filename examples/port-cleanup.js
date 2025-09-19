// Port Cleanup Script
// Automatically cleans up development ports

log("Starting port cleanup script")

// Clean up common development ports
clearPort(3000)
clearPort(3001)
clearPort(5000)
clearPort(8000)
clearPort(8080)
clearPort(9000)

log("Port cleanup completed")

// Monitor for new processes
onPort(3000, callback)
onPort(8080, callback)

log("Monitoring active - press Ctrl+C to stop")
