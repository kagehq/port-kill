// Simple Port Guard Script
// Automatically kills any process that binds to port 3000

log("Starting simple port guard for port 3000");

// Guard port 3000 - kill any process that tries to use it
guardPort(3000);

log("Port guard activated. Any process on port 3000 will be killed.");
