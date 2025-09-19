// File Cleanup Script
// Kills processes with common problematic files open

log("Starting file cleanup script...");

// Kill processes with lock files open
killFileExt(".lock");

// Kill processes with log files open
killFileExt(".log");

// Kill processes with specific files open
killFile("package-lock.json");
killFile("yarn.lock");

log("File cleanup completed. Lock and log files should be free.");
