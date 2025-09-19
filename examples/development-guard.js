// Development Environment Guard Script
// Protects your development environment from file conflicts

log("Starting development environment guard...");

// Guard critical development files
guardFile("package.json");
guardFile("package-lock.json");
guardFile(".env");

// Kill processes with lock files that might cause conflicts
killFileExt(".lock");

// Guard your main development port
guardPort(3000);

log("Development environment guard activated:");
log("  - package.json: Protected");
log("  - package-lock.json: Protected");
log("  - .env: Protected");
log("  - .lock files: Cleared");
log("  - Port 3000: Guarded");
