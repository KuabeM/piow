//! Rename workspaces of your sway window manager by dynamically adding icons of applications in
//! each workspace. Configuration is done in toml file at `${XDG_CONFIG_HOME}/piow/config.toml`.
use failure::Error;
use futures_util::stream::StreamExt;
use serde_derive::Deserialize;
use std::path::PathBuf;
use swayipc_async::{Connection, Event, EventType, WorkspaceChange};

mod config;
mod nodes;

/// Docopt usage string.
const USAGE: &str = r#"
Put icons of apps on sway workspaces.

Usage:
    piow [--config=<cfg>]
    piow (-h|--help)
    piow (-v|--version)

Options:
  -h --help            Show this screen.
  --version            Print the version and exit.
  --config=<cfg>       Path to config file. Defaults to $XDG_CONFIG_HOME or $HOME/.config.
"#;

/// Docopt argument struct.
#[derive(Debug, Deserialize, Default)]
struct Args {
    flag_config: Option<PathBuf>,
    flag_version: bool,
    flag_help: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .init();

    if args.flag_help {
        println!("{}", USAGE);
        return Ok(());
    } else if args.flag_version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Load config: app_id to icon mapping
    let cfg = config::Config::load(args.flag_config).unwrap_or_default();

    // Subscribe to sway events, only Workspace event is interesting
    let subs = [EventType::Workspace];
    let connection = Connection::new().await?;
    let mut events = connection.subscribe(&subs).await?;
    // Get connections for sending commands
    let mut cmd_curr_con = Connection::new().await?;
    let mut cmd_old_con = Connection::new().await?;

    while let Some(event) = events.next().await {
        let curr_ws = match event? {
            Event::Workspace(ev) => {
                if ev.change != WorkspaceChange::Focus {
                    log::trace!("Event '{:?}' not processed.", ev.change);
                    continue;
                }
                log::trace!("New event: '{:?}'", ev.change);
                ev.current.unwrap()
            }
            _ => unreachable!("Unsubscribed events unreachable."),
        };
        // Get new name for current workspace (The one we landed on).
        let (name_curr, cmd_curr) = match nodes::construct_rename_cmd(&curr_ws, &cfg) {
            Some(cmd) => cmd,
            None => continue,
        };
        // Get new name for the old workspace (The one we started on).
        let (name_old, cmd_old) = match nodes::construct_rename_cmd(&curr_ws, &cfg) {
            Some(cmd) => cmd,
            None => continue,
        };
        // Run the commands
        let cmd_curr_res = cmd_curr_con.run_command(&cmd_curr);
        let cmd_old_res = cmd_old_con.run_command(&cmd_old);
        for outcome in cmd_curr_res.await? {
            if let Err(error) = outcome {
                log::error!("Failed to rename workspace '{}': '{}'", name_curr, error);
            }
        }
        for outcome in cmd_old_res.await? {
            if let Err(error) = outcome {
                log::error!("Failed to rename workspace '{}': '{}'", name_old, error);
            }
        }
    }
    Ok(())
}
