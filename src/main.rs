#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
use std::path::PathBuf;

use anyhow::{Result, bail};
use chrono::Local;
use clap::{Parser, Subcommand};
use clickpulse::{
    db::{EventDb, Snapshot},
    source::LiveSource,
};

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
mod desktop;

#[derive(Parser)]
#[command(
    version = clickpulse::version::DISPLAY,
    about = "ClickPulse · Local keyboard and mouse activity statistics"
)]
struct Cli {
    #[arg(long, hide = true)]
    apply_update: Option<PathBuf>,
    /// Record independently to an explicitly selected database
    #[arg(long, global = true, conflicts_with = "source_db")]
    db: Option<PathBuf>,
    /// Read an explicitly selected existing database without changing it
    #[arg(long, global = true, conflicts_with = "db")]
    source_db: Option<PathBuf>,
    /// Import an explicitly selected legacy database once into --db
    #[arg(long, global = true, requires = "db")]
    legacy_db: Option<PathBuf>,
    /// Start paused (pause refresh when reading an existing source)
    #[arg(long)]
    paused: bool,
    /// Preview synthetic data without opening a database or capturing input
    #[arg(long)]
    demo: bool,
    /// Start in the menu bar or system tray
    #[arg(long)]
    background: bool,
    /// Open mini mode while continuing background recording
    #[arg(long)]
    mini: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show statistics without starting input capture
    Stats {
        #[arg(long)]
        json: bool,
    },
    /// Import a legacy database into the independently selected --db
    Import { path: PathBuf },
    /// Migrate an existing database to a new file without stopping any service
    Migrate {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Legacy encoding change timestamp with UTC offset (RFC3339)
        #[arg(long)]
        legacy_key_cutoff: Option<chrono::DateTime<chrono::FixedOffset>>,
    },
}

fn print_stats(stats: Snapshot, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("ClickPulse · {}", stats.now);
        for (name, count) in [
            ("Today", stats.today),
            ("Last 7 days", stats.week),
            ("All time", stats.lifetime),
        ] {
            println!(
                "{name}: {} (keys {} / clicks {})",
                count.total(),
                count.keyboard,
                count.mouse
            );
        }
        for window in stats.windows {
            println!(
                "Last {}s: {}{}",
                window.seconds,
                window.counts.total(),
                if window.approximate {
                    " (historical detail approximated by minute)"
                } else {
                    ""
                }
            );
        }
    }
    Ok(())
}

fn main() {
    #[cfg(target_os = "windows")]
    clickpulse::platform::windows::attach_parent_console();
    let cli = Cli::parse();
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    let desktop_mode = cli.command.is_none() && cli.apply_update.is_none();
    if let Err(error) = run(cli) {
        eprintln!("ClickPulse: {error:#}");
        #[cfg(target_os = "windows")]
        if desktop_mode {
            clickpulse::platform::windows::show_startup_error(&error);
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        if desktop_mode {
            rfd::MessageDialog::new()
                .set_title("ClickPulse")
                .set_description(format!("{error:#}"))
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    if let Some(plan) = &cli.apply_update {
        return clickpulse::updater::apply_update(plan);
    }
    #[cfg(target_os = "macos")]
    if cli.command.is_none() && !cli.demo {
        clickpulse::handoff::recover_pending()?;
    }
    if let Some(Commands::Migrate {
        source,
        output,
        legacy_key_cutoff,
    }) = &cli.command
    {
        let cutoff = legacy_key_cutoff.map(|date| date.timestamp().div_euclid(60));
        let stats =
            clickpulse::collector::CollectorDb::migrate_with_key_cutoff(source, output, cutoff)?;
        println!(
            "Migrated {} operations across {} recorded days to {}",
            stats.lifetime.total(),
            stats.history_days.len(),
            output.display()
        );
        return Ok(());
    }
    // A fresh installation always owns its platform user-data directory.
    // Legacy databases are accessed only through explicit CLI arguments.
    let source_mode = cli.source_db.is_some();
    let owned = cli.db.is_none() && !source_mode;
    let setup_pending = false;
    let path = match cli.db.or(cli.source_db) {
        Some(path) => path,
        None if cli.demo => PathBuf::new(),
        None => clickpulse::collector::data_path()?,
    };
    if let Some(command) = cli.command {
        if cli.demo {
            bail!("--demo is only available for desktop previews");
        }
        if source_mode || owned {
            if matches!(command, Commands::Import { .. }) {
                bail!(
                    "Import requires an explicit --db destination; the default App database is managed by the desktop recorder"
                );
            }
            let source = LiveSource::open(&path)?;
            match command {
                Commands::Stats { json } => print_stats(source.snapshot(Local::now())?, json)?,
                Commands::Import { .. } => unreachable!(),
                Commands::Migrate { .. } => unreachable!(),
            }
        } else {
            let mut db = EventDb::open(&path)?;
            if let Some(legacy) = cli.legacy_db.as_deref() {
                db.import_legacy(legacy)?;
            }
            match command {
                Commands::Stats { json } => print_stats(db.snapshot(Local::now())?, json)?,
                Commands::Import { path: source } => {
                    if source.canonicalize()? == path.canonicalize()? {
                        bail!("The legacy source and destination database must be different files");
                    }
                    let count = db.import_legacy(&source)?;
                    println!(
                        "Imported {count} historical operations into {}",
                        path.display()
                    );
                }
                Commands::Migrate { .. } => unreachable!(),
            }
        }
        return Ok(());
    }
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    return desktop::run(
        &path,
        cli.legacy_db.as_deref(),
        cli.paused,
        cli.demo,
        desktop::Options {
            source_mode,
            owned,
            setup_pending,
            background: cli.background,
            mini: cli.mini,
        },
    );
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    bail!(
        "The desktop application requires macOS or Windows; use stats, import or migrate instead"
    );
}
