//! Extract application ids from sway workspace tree and map them to workspace names.
use log::{debug, trace, warn};
use swayipc_async::Node;

use crate::config::Config;

fn filter_nodes(node: &Node) -> Vec<String> {
    let mut ids: Vec<String> = node
        .nodes
        .iter()
        .flat_map(|n| {
            let mut ids = filter_nodes(n);
            if let Some(props) = n.window_properties.as_ref() {
                if let Some(class) = props.class.as_ref() {
                    ids.push(class.to_string());
                }
            }
            if let Some(id) = n.app_id.as_ref() {
                ids.push(id.to_string());
            }
            ids
        })
        .collect();
    if let Some(app_id) = &node.app_id {
        ids.push(app_id.clone());
    }
    ids
}

/// Collection of App Ids scraped from sway workspace tree.
#[derive(Debug)]
pub struct AppIds {
    inner: Vec<String>,
}

impl From<&Node> for AppIds {
    fn from(workspace: &Node) -> Self {
        let mut ids = filter_nodes(workspace);
        trace!("App ids: {:?}", ids);

        let mut floating = workspace
            .floating_nodes
            .iter()
            .flat_map(filter_nodes)
            .collect();
        trace!("Floating app ids {:?}", floating);
        ids.append(&mut floating);

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
    /// Resulting string is the new workspace name consisting of workspace number, separator and
    /// icons.
    pub fn map(&self, cfg: &Config) -> String {
        let mut icons: Vec<String> = self
            .inner
            .iter()
            .filter_map(|id| {
                if let Some(ics) = cfg
                    .icons
                    .keys()
                    .find(|e| id.to_lowercase().contains(&e.to_string()))
                {
                    trace!("Found icon {:?} for id '{}'", cfg.icons.get(ics), &id);
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
pub fn construct_rename_cmd(workspace: &Node, cfg: &Config) -> Option<(String, String)> {
    let ws_name = match workspace.name {
        Some(ref n) => n,
        None => return None,
    };
    let ws_num = match workspace.num {
        Some(ref s) => s,
        None => return None,
    };

    // Get icons to place on current workspace
    let icons = AppIds::from(workspace).map(cfg);
    let format = cfg.format(ws_num.to_string(), icons);

    let cmd = "rename workspace '".to_string() + ws_name + "' to '" + &format + "'";
    log::trace!("Cmd: >{}<", cmd);
    Some((ws_name.to_string(), cmd))
}
