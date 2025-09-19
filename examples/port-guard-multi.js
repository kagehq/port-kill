// Multi-Port Guard Script
// Guards multiple ports with different policies

log("Starting multi-port guard system");

// Guard port 3000 - kill any process
guardPort(3000);

// Guard port 8080 - only allow "nginx"
guardPort(8080, "nginx");

// Guard port 9000 - kill any process
guardPort(9000);

log("Multi-port guard activated:");
log("  - Port 3000: Kill all processes");
log("  - Port 8080: Only allow 'nginx'");
log("  - Port 9000: Kill all processes");
