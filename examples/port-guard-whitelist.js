// Port Guard with Whitelist Script
// Only allows specific processes on port 3000, kills everything else

log("Starting port guard with whitelist for port 3000");

// Only allow "my-dev-server" on port 3000, kill everything else
guardPort(3000, "my-dev-server");

log("Port guard activated. Only 'my-dev-server' is allowed on port 3000.");
