use anyhow::Result;
use log::info;
use port_kill::{app::PortKillApp, cli::Args};
use clap::Parser;

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Set up logging level based on log_level argument
    let log_level = if args.verbose {
        // Verbose flag overrides log_level for backward compatibility
        "debug"
    } else {
        args.log_level.to_rust_log()
    };
    std::env::set_var("RUST_LOG", log_level);

    // Initialize logging
    env_logger::init();
    
    info!("Starting Port Kill application...");
    info!("Monitoring: {}", args.get_port_description());

    // Create and run the application
    let app = PortKillApp::new(args)?;
    app.run()?;

    info!("Port Kill application stopped");
    Ok(())
}
