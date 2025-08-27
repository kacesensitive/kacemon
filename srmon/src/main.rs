use clap::{Arg, Command};
use kacemon_core::Config;
use std::{io::stdout, path::PathBuf, process};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    // Parse command line arguments
    let matches = Command::new("kacemon")
        .version("0.2.0")
        .about("System Resource Monitor - A cross-platform TUI system monitor")
        .arg(
            Arg::new("refresh")
                .long("refresh")
                .value_name("MS")
                .help("Refresh interval in milliseconds")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("theme")
                .long("theme")
                .value_name("THEME")
                .help("UI theme")
                .value_parser(["dark", "light"])
        )
        .arg(
            Arg::new("no-color")
                .long("no-color")
                .help("Disable colors")
                .action(clap::ArgAction::SetTrue)
        )

        .arg(
            Arg::new("json-config")
                .long("json-config")
                .value_name("PATH")
                .help("Path to JSON configuration file")
                .value_parser(clap::value_parser!(PathBuf))
        )
        .get_matches();

    // Build CLI configuration
    let cli_config = kacemon_core::config::CliConfig {
        refresh_ms: matches.get_one::<u64>("refresh").copied(),
        theme: matches.get_one::<String>("theme").map(|t| match t.as_str() {
            "light" => kacemon_core::Theme::Light,
            _ => kacemon_core::Theme::Dark,
        }),
        no_color: matches.get_flag("no-color"),
    };

    // Load configuration
    let json_config_path = matches.get_one::<PathBuf>("json-config");
    let config = Config::load(Some(&cli_config), json_config_path)?;

    // Validate refresh rate
    if config.refresh_ms < 100 {
        eprintln!("Warning: Refresh rate too low, using 100ms minimum");
    }

    // Run TUI
    run_tui_only(config)
}

/// Run in TUI-only mode
fn run_tui_only(config: Config) -> anyhow::Result<()> {
    let mut app = kacemon_tui::App::new(config)?;
    let mut stdout = stdout();
    app.run(&mut stdout)?;
    Ok(())
}
