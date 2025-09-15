use anyhow::Result;
use log::info;
use port_kill::{console_app::ConsolePortKillApp, cli::Args};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();
    
    // Validate arguments
    if let Err(e) = args.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Set up logging level based on verbose flag
    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Initialize logging
    env_logger::init();
    
    info!("Starting Console Port Kill application...");
    info!("Monitoring: {}", args.get_port_description());

    // Handle special commands
    if args.show_history {
        let app = ConsolePortKillApp::new(args)?;
        app.display_history().await?;
        return Ok(());
    }
    
    if args.clear_history {
        let app = ConsolePortKillApp::new(args)?;
        app.clear_history().await?;
        return Ok(());
    }
    
    if args.show_filters {
        let app = ConsolePortKillApp::new(args)?;
        app.display_filter_info().await?;
        return Ok(());
    }
    
    if args.kill_all {
        let app = ConsolePortKillApp::new(args)?;
        app.kill_all_processes().await?;
        return Ok(());
    }
    
    if let Some(ref groups) = args.kill_group {
        let groups: Vec<String> = groups.clone();
        let app = ConsolePortKillApp::new(args)?;
        app.kill_by_group(&groups).await?;
        return Ok(());
    }
    
    if let Some(ref projects) = args.kill_project {
        let projects: Vec<String> = projects.clone();
        let app = ConsolePortKillApp::new(args)?;
        app.kill_by_project(&projects).await?;
        return Ok(());
    }
    
    if args.restart {
        let app = ConsolePortKillApp::new(args)?;
        app.restart_processes().await?;
        return Ok(());
    }
    
    if args.show_tree {
        let app = ConsolePortKillApp::new(args)?;
        app.show_process_tree().await?;
        return Ok(());
    }

    // Create and run the console application
    let app = ConsolePortKillApp::new(args)?;
    app.run().await?;

    info!("Console Port Kill application stopped");
    Ok(())
}
