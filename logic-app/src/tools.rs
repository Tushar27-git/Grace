use eframe::egui::Pos2;
use logic_core::{ComponentId, NetId, PortEndpoint};
use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTool {
    #[default]
    Select,
    Wire,
}

#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub selected_components: HashSet<ComponentId>,
    pub selected_nets: HashSet<NetId>,
}

impl SelectionState {
    pub fn is_empty(&self) -> bool {
        self.selected_components.is_empty() && self.selected_nets.is_empty()
    }

    pub fn clear(&mut self) {
        self.selected_components.clear();
        self.selected_nets.clear();
    }

    pub fn select_single(&mut self, id: ComponentId) {
        self.selected_components.clear();
        self.selected_nets.clear();
        self.selected_components.insert(id);
    }

    pub fn toggle_component(&mut self, id: ComponentId) {
        if self.selected_components.contains(&id) {
            self.selected_components.remove(&id);
        } else {
            self.selected_components.insert(id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct WireInProgress {
    pub source_endpoint: PortEndpoint,
    pub source_pos: Pos2,
    pub current_cursor: Pos2,
}

#[derive(Debug, Clone)]
pub struct MarqueeState {
    pub start: Pos2,
    pub current: Pos2,
}
