use log::{debug, trace, warn};
use swayipc_async::Node;

use crate::config::Config;

fn strip(input: &String, patterns: &[&str]) -> String {
    let mut work = input.clone();
    for p in patterns.iter() {
        trace!("replace >{}< in >{}<", p, work);
        work = work.replace(p, "");
    }
    work
}

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
            .map(|id| id.clone())
            .collect();
        let mut floating: Vec<String> = workspace
            .floating_nodes
            .iter()
            .filter_map(|n| n.app_id.as_ref())
            .map(|id| id.clone())
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
    pub fn map<'a>(&self, cfg: &Config) -> Vec<String> {
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
            .map(|e| e.clone())
            .collect();
        icons.sort();
        icons.dedup();
        debug!("Found icons '{:?}' for ids '{:?}'", icons, self.inner);
        icons
    }
}
