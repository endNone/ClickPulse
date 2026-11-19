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
    /// Remove ClickPulse on Windows, with an explicit choice about saved data
    #[cfg(windows)]
    Uninstall {
        #[arg(long, hide = true)]
        from: Option<PathBuf>,
    },
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
    #[cfg(windows)]
    let removal_mode = matches!(cli.command, Some(Commands::Uninstall { .. }));
    #[cfg(windows)]
    let desktop_mode = desktop_mode || removal_mode;
    if let Err(error) = run(cli) {
        eprintln!("ClickPulse: {error:#}");
        #[cfg(target_os = "windows")]
        if desktop_mode {
            clickpulse::platform::windows::show_error(
                &error,
                if removal_mode {
                    "error.removal"
                } else {
                    "error.startup"
                },
            );
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        if desktop_mode {
            clickpulse::dialogs::MessageDialog::new()
                .set_title("ClickPulse")
                .set_description(format!(
                    "{}\n\n{error:#}",
                    clickpulse::presentation::Preferences::locale_for_error().text("error.startup")
                ))
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    #[cfg(windows)]
    if let Some(Commands::Uninstall { from }) = &cli.command {
        anyhow::ensure!(
            cli.db.is_none() && cli.source_db.is_none() && !cli.demo && cli.apply_update.is_none(),
            "Uninstall cannot be combined with database or update options"
        );
        return match from {
            Some(original) => clickpulse::removal::run(original),
            None => clickpulse::removal::launch(),
        };
    }
    if let Some(plan) = &cli.apply_update {
        return clickpulse::updater::apply_update(plan);
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
        // Desktop-owned storage is resolved only after acquiring the profile
        // lock, so recovery cannot rename files beneath another running App.
        None if cli.demo || cli.command.is_none() => PathBuf::new(),
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
            if owned
                && !path.try_exists()?
                && let Commands::Stats { json } = command
            {
                return print_stats(EventDb::empty_snapshot(Local::now())?, json);
            }
            let source = LiveSource::open(&path)?;
            match command {
                Commands::Stats { json } => print_stats(source.snapshot(Local::now())?, json)?,
                Commands::Import { .. } => unreachable!(),
                Commands::Migrate { .. } => unreachable!(),
                #[cfg(windows)]
                Commands::Uninstall { .. } => unreachable!(),
            }
        } else {
            anyhow::ensure!(
                !(matches!(command, Commands::Import { .. }) && cli.legacy_db.is_some()),
                "Use either import or --legacy-db, not both"
            );
            let _write_lock =
                if matches!(command, Commands::Import { .. }) || cli.legacy_db.is_some() {
                    Some(clickpulse::storage::lock_database(&path)?)
                } else {
                    None
                };
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
                #[cfg(windows)]
                Commands::Uninstall { .. } => unreachable!(),
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
