// File Guard with Whitelist Script
// Only allows specific processes to open certain files

log("Starting file guard with whitelist for .env files");

// Only allow "npm" to open .env files, kill everything else
guardFile(".env", "npm");

log("File guard activated. Only 'npm' is allowed to open .env files.");
