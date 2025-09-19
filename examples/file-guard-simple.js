// Simple File Guard Script
// Automatically kills any process that opens a specific file

log("Starting simple file guard for package.json");

// Guard package.json - kill any process that opens it
guardFile("package.json");

log("File guard activated. Any process that opens package.json will be killed.");
