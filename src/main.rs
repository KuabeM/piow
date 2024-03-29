//! Rename workspaces of your sway window manager by dynamically adding icons of applications in
//! each workspace. Configuration is done in toml file at `${XDG_CONFIG_HOME}/piow/config.toml`.
use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use log::error;
use serde_derive::Deserialize;
use std::path::PathBuf;
use swayipc_async::{Connection, Event, EventType, WorkspaceChange};
use sysinfo::{ProcessExt, SystemExt};

mod config;
mod nodes;

/// Docopt usage string.
const USAGE: &str = r#"
Put icons of apps on sway workspaces.

Usage:
    piow [--config=<cfg>] [--syslog] [--replace]
    piow (-h|--help)
    piow (-v|--version)

Options:
  -h --help            Show this screen.
  --version            Print the version and exit.
  --config=<cfg>       Path to config file. Defaults to piow/config.toml under
                        $XDG_CONFIG_HOME, $HOME/.config, or /etc/xdg.
  --syslog             Send log messages to syslog instead of stdout.
  --replace            Replace currently running piow instances.
"#;

/// Docopt argument struct.
#[derive(Debug, Deserialize, Default)]
struct Args {
    flag_config: Option<PathBuf>,
    flag_syslog: bool,
    flag_version: bool,
    flag_help: bool,
    flag_replace: bool,
}

macro_rules! skip_none {
    ($f:expr) => {
        match $f {
            Some(s) => s,
            None => continue,
        }
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_syslog {
        syslog::init(
            syslog::Facility::LOG_USER,
            log::LevelFilter::Warn,
            Some(env!("CARGO_PKG_NAME")),
        )
        .map_err(|e| anyhow!("{}", e))?;
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .format_timestamp(None)
            .init();
    }

    if args.flag_help {
        println!("{}", USAGE);
        return Ok(());
    } else if args.flag_version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.flag_replace {
        let mut sys = sysinfo::System::new();
        sys.refresh_processes();
        let own_pid = sysinfo::get_current_pid().map_err(|e| anyhow!(e))?;
        for proc in sys
            .processes_by_name(env!("CARGO_PKG_NAME"))
            .filter(|proc| proc.pid() != own_pid)
        {
            log::info!("Replacing piow instance {}", proc.pid());
            proc.kill_with(sysinfo::Signal::Quit)
                .ok_or(anyhow!("Failed to kill {}", proc.name()))?;
        }
    }

    // Load config: app_id to icon mapping
    let cfg = config::Config::load(args.flag_config)
        .map_err(|e| error!("{}", e))
        .unwrap_or_default();

    // Subscribe to sway events, only Workspace event is interesting
    let subs = [EventType::Workspace];
    let connection = Connection::new().await?;
    let mut events = connection.subscribe(&subs).await?;
    // Get connection for sending commands
    let mut cmd_con = Connection::new().await?;

    while let Some(event) = events.next().await {
        match event? {
            Event::Workspace(ev) => {
                if ev.change != WorkspaceChange::Focus {
                    log::trace!("Workspace Event '{:?}' not processed.", ev.change);
                    continue;
                }
                log::trace!("New workspace event: '{:?}'", ev.change);
                // Get new name for current workspace (The one we landed on).
                let (name_curr, cmd_curr) =
                    skip_none!(nodes::construct_rename_cmd(&ev.current.unwrap(), &cfg));

                // Get new name for old workspace (The one we came from).
                let (name_old, cmd_old) =
                    skip_none!(nodes::construct_rename_cmd(&ev.old.unwrap(), &cfg));
                // Run the commands
                cmd_con
                    .run_command(&cmd_curr)
                    .await?
                    .iter()
                    .filter(|r| r.is_err())
                    .for_each(|e| {
                        log::debug!(
                            "Failed to rename workspace '{}': '{}'",
                            name_curr,
                            e.as_ref().unwrap_err()
                        );
                    });

                cmd_con
                    .run_command(&cmd_old)
                    .await?
                    .iter()
                    .filter(|r| r.is_err())
                    .for_each(|e| {
                        log::debug!(
                            "Failed to rename workspace '{}': '{}'",
                            name_old,
                            e.as_ref().unwrap_err()
                        );
                    });
            }
            _ => unreachable!("Unsubscribed events unreachable."),
        };
    }
    Ok(())
}
