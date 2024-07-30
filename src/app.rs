use std::collections::BTreeMap;
use std::sync::Arc;

use crate::plugin::Plugin;

#[derive(Clone, Debug)]
pub struct App {
    plugins: BTreeMap<String, Arc<dyn Plugin>>,
}

impl App {
    pub fn new(plugins: BTreeMap<String, Arc<dyn Plugin>>) -> Arc<Self> {
        Arc::new(Self { plugins })
    }

    pub fn get_plugin(&self, tag: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(tag).cloned()
    }
}
