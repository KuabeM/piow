use failure::Error;
use futures_util::stream::StreamExt;
use std::path::PathBuf;
use swayipc_async::{Connection, Event, EventType};
use serde_derive::Deserialize;

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

    while let Some(event) = events.next().await {
        let curr_ws = match event? {
            Event::Workspace(ev) => ev.current.unwrap(),
            _ => unreachable!("Unsubscribed events unreachable"),
        };
        let ws_name = match curr_ws.name {
            Some(ref n) => n,
            None => continue,
        };
        let ws_num = match curr_ws.num {
            Some(ref s) => s,
            None => continue,
        };

        // Get icons to place on current workspace
        let icons = nodes::AppIds::from(&curr_ws).map(&cfg).join(" ");
        let format = cfg.format(ws_num.to_string(), icons);

        let cmd = "rename workspace '".to_string()
            + &ws_name
            + "' to '"
            + &format
            + "'";
        log::trace!("Cmd: >{}<", cmd);
        let mut connection2 = Connection::new().await?;
        for outcome in connection2.run_command(&cmd).await? {
            if let Err(error) = outcome {
                log::error!("Failed to rename workspace '{}': '{}'", ws_name, error);
            }
        }
    }
    Ok(())
}
