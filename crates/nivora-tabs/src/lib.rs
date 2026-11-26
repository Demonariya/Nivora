//! Small in-memory tab model. Native WKView ownership is kept by nivora-engine.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub is_private: bool,
}

#[derive(Debug, Default)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active: Option<TabId>,
    next_id: u64,
}

impl TabManager {
    pub fn open(&mut self, url: impl Into<String>, is_private: bool) -> TabId {
        self.next_id += 1;
        let id = TabId(self.next_id);
        self.tabs.push(Tab {
            id,
            title: "New Tab".into(),
            url: url.into(),
            is_private,
        });
        self.active = Some(id);
        id
    }

    pub fn close(&mut self, id: TabId) -> bool {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return false;
        };
        self.tabs.remove(index);
        if self.active == Some(id) {
            self.active = self.tabs.get(index).or_else(|| self.tabs.last()).map(|tab| tab.id);
        }
        true
    }

    pub fn active(&self) -> Option<&Tab> {
        self.tabs.iter().find(|tab| Some(tab.id) == self.active)
    }

    pub fn len(&self) -> usize { self.tabs.len() }
    pub fn is_empty(&self) -> bool { self.tabs.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::TabManager;

    #[test]
    fn opening_and_closing_tabs_updates_active_tab() {
        let mut tabs = TabManager::default();
        let first = tabs.open("about:blank", false);
        let second = tabs.open("https://example.com", true);
        assert_eq!(tabs.active().unwrap().id, second);
        assert!(tabs.close(second));
        assert_eq!(tabs.active().unwrap().id, first);
        assert_eq!(tabs.len(), 1);
    }
}
