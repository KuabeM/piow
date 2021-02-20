//! Extract application ids from sway workspace tree and map them to workspace names.
use log::{debug, trace, warn};
use swayipc_async::Node;

use crate::config::Config;

/// Strip a slice of patterns from a string.
fn strip(input: &str, patterns: &[&str]) -> String {
    let mut work = input.to_string();
    for p in patterns.iter() {
        trace!("replace >{}< in >{}<", p, work);
        work = work.replace(p, "");
    }
    work
}

/// Collection of App Ids scraped from sway workspace tree.
#[derive(Debug)]
pub struct AppIds {
    inner: Vec<String>,
}

impl From<&Node> for AppIds {
    fn from(workspace: &Node) -> Self {
        let mut ids: Vec<String> = workspace
            .nodes
            .iter()
            .filter_map(|n| n.app_id.as_ref())
            .cloned()
            .collect();
        let mut floating: Vec<String> = workspace
            .floating_nodes
            .iter()
            .filter_map(|n| n.app_id.as_ref())
            .cloned()
            .collect();
        ids.append(&mut floating);
        if let Some(reps) = workspace.representation.as_ref() {
            let mut id: Vec<String> = strip(reps, &["H[", "V[", "[", "]", "\""])
                .split(' ')
                .map(|i| i.to_string())
                .collect();
            debug!("Found no app ids, but ids in the representation {:?}", id);
            ids.append(&mut id);
        }
        debug!(
            "Found app ids '{:?}' on workspace '{}'",
            ids,
            workspace.name.as_ref().unwrap_or(&"<unnamed>".to_string())
        );
        Self { inner: ids }
    }
}

impl AppIds {
    /// Map collected App Ids to a new workspace name based on the config.
    ///
    /// Resulting string is the new workspace name consisting of workspace number, separator and icons.
    pub fn map(&self, cfg: &Config) -> String {
        let mut icons: Vec<String> = self
            .inner
            .iter()
            .filter_map(|id| {
                trace!("Looking for id '{}'", &id);
                if let Some(ics) = cfg.icons.keys().find(|e| e.contains(&id.to_lowercase())) {
                    cfg.icons.get(ics)
                } else {
                    warn!("No icon for application '{}' in the config.", id);
                    Some(&cfg.default_icon)
                }
            })
            .cloned()
            .collect();
        icons.sort();
        icons.dedup();
        debug!("Found icons '{:?}' for ids '{:?}'", icons, self.inner);
        icons.join(&cfg.icon_separator)
    }
}

/// Construct the sway command for renaming `workspace`.
///
/// Returns a tuple of workspace name and command for renaming.
pub fn contruct_rename_cmd(workspace: &Node, cfg: &Config) -> Option<(String, String)> {
    let ws_name = match workspace.name {
        Some(ref n) => n,
        None => return None,
    };
    let ws_num = match workspace.num {
        Some(ref s) => s,
        None => return None,
    };

    // Get icons to place on current workspace
    let icons = AppIds::from(workspace).map(&cfg);
    let format = cfg.format(ws_num.to_string(), icons);

    let cmd = "rename workspace '".to_string() + &ws_name + "' to '" + &format + "'";
    log::trace!("Cmd: >{}<", cmd);
    Some((ws_name.to_string(), cmd))
}
