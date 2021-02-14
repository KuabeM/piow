use swayipc_async::Node;

#[derive(Debug)]
pub struct SwayNodes {
    nodes: Vec<SwayNodes>,
    app_id: Option<String>,
}

impl From<&Node> for SwayNodes {
    fn from(node: &Node) -> Self {
        let nodes = node
            .nodes
            .iter()
            .map(|n| {
                let app_id = n.app_id.as_ref();
                let inner: Self = match app_id {
                    Some(id) => Self {
                        nodes: Vec::new(),
                        app_id: Some(id.clone()),
                    },
                    None => Self::from(n),
                };
                inner
            })
            .collect();
        let app_id = match node.app_id.as_ref() {
            Some(s) => Some(s.clone()),
            None => None
        };

        Self {
            nodes,
            app_id,
        }
    }
}

impl SwayNodes {
    pub fn flatten(&self) -> Vec<String> {
        let mut acc = self
            .nodes
            .iter()
            .fold(Vec::new(), |mut acc: Vec<String>, s| {
                if let Some(id) = s.app_id.as_ref() {
                    acc.push(id.clone());
                }
                let mut inner: Vec<String> = s.flatten();
                acc.append(&mut inner);
                acc
            });
        if let Some(id) = self.app_id.as_ref() {
            acc.push(id.to_string());
        }
        acc
    }
}
