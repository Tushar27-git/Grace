use eframe::egui::Pos2;
use logic_core::{ComponentId, NetId, PortEndpoint};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTool {
    #[default]
    Normal,   // 4th: Normal view mode (+2% snap radius, full edit/move/wire)
    Marquee,  // 1st: Marquee/selection tool (only selects components)
    Connect,  // 2nd: Connection tool (+5% snap radius, dedicated wiring)
    Pan,      // 3rd: Pan/move around tool (drag canvas)
}

impl ActiveTool {
    pub fn label(&self) -> &'static str {
        match self {
            ActiveTool::Normal => "Normal (View & Edit)",
            ActiveTool::Marquee => "Marquee Select",
            ActiveTool::Connect => "Connection (+5% Snap)",
            ActiveTool::Pan => "Pan / Move Canvas",
        }
    }

    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        match self {
            ActiveTool::Normal => "PTR",
            ActiveTool::Marquee => "BOX",
            ActiveTool::Connect => "WIRE",
            ActiveTool::Pan => "PAN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlacementMode {
    #[default]
    Single,
    Multi,
}

impl PlacementMode {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            PlacementMode::Single => "Single",
            PlacementMode::Multi => "Multi",
        }
    }
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

#[derive(Debug, Clone)]
pub struct WireDropSearchState {
    pub source_endpoint: PortEndpoint,
    pub source_canvas_pos: Pos2,
    pub drop_canvas_pos: Pos2,
    pub search_query: String,
    pub selected_index: usize,
    pub request_focus: bool,
}
