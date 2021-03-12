//! Rename workspaces of your sway window manager by dynamically adding icons of applications in
//! each workspace. Configuration is done in toml file at `${XDG_CONFIG_HOME}/piow/config.toml`.
use failure::{format_err, Error};
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
    piow [--config=<cfg>] [--syslog]
    piow (-h|--help)
    piow (-v|--version)

Options:
  -h --help            Show this screen.
  --version            Print the version and exit.
  --config=<cfg>       Path to config file. Defaults to $XDG_CONFIG_HOME or $HOME/.config.
  --syslog             Send log messages to syslog instead of stdout.
"#;

/// Docopt argument struct.
#[derive(Debug, Deserialize, Default)]
struct Args {
    flag_config: Option<PathBuf>,
    flag_syslog: bool,
    flag_version: bool,
    flag_help: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_syslog {
        syslog::init(
            syslog::Facility::LOG_USER,
            log::LevelFilter::Warn,
            Some(env!("CARGO_PKG_NAME")),
        )
        .map_err(|e| format_err!("{}", e))?;
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

    // Load config: app_id to icon mapping
    let cfg = config::Config::load(args.flag_config).unwrap_or_default();

    // Subscribe to sway events, only Workspace event is interesting
    let subs = [EventType::Workspace];
    let connection = Connection::new().await?;
    let mut events = connection.subscribe(&subs).await?;
    // Get connection for sending commands
    let mut cmd_con = Connection::new().await?;

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
        // Run the command
        let cmd_res = cmd_con.run_command(&cmd_curr);
        for outcome in cmd_res.await? {
            if let Err(error) = outcome {
                log::debug!("Failed to rename workspace '{}': '{}'", name_curr, error);
            }
        }
    }
    Ok(())
}
