use crate::{
    gui::{
        Layout,
        UserInterface,
        node::{UINode, UINodeKind},
        builder::{
            CommonBuilderFields,
            GenericNodeBuilder
        },
    },
};

use rg3d_core::{
    pool::Handle,
    math::{
        vec2::Vec2,
        Rect
    }
};
use crate::gui::EventSource;
use crate::gui::event::UIEvent;

pub struct Canvas {
    pub(in crate::gui) owner_handle: Handle<UINode>
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            owner_handle: Handle::NONE
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Layout for Canvas {
    fn measure_override(&self, ui: &UserInterface, _available_size: Vec2) -> Vec2 {
        let size_for_child = Vec2::make(
            std::f32::INFINITY,
            std::f32::INFINITY,
        );

        if let Some(node) = ui.nodes.borrow(self.owner_handle) {
            for child_handle in node.children.iter() {
                ui.measure(*child_handle, size_for_child);
            }
        }

        Vec2::zero()
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: Vec2) -> Vec2 {
        if let Some(node) = ui.nodes.borrow(self.owner_handle) {
            for child_handle in node.children.iter() {
                let mut final_rect = None;

                if let Some(child) = ui.nodes.borrow(*child_handle) {
                    final_rect = Some(Rect::new(
                        child.desired_local_position.get().x,
                        child.desired_local_position.get().y,
                        child.desired_size.get().x,
                        child.desired_size.get().y));
                }

                if let Some(rect) = final_rect {
                    ui.arrange(*child_handle, &rect);
                }
            }
        }

        final_size
    }
}

pub struct CanvasBuilder {
    common: CommonBuilderFields
}

impl Default for CanvasBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasBuilder {
    pub fn new() -> Self {
        Self {
            common: CommonBuilderFields::new()
        }
    }

    impl_default_builder_methods!();

    pub fn build(self, ui: &mut UserInterface) -> Handle<UINode> {
        GenericNodeBuilder::new(UINodeKind::Canvas(Canvas::new()), self.common).build(ui)
    }
}

impl EventSource for Canvas {
    fn emit_event(&mut self) -> Option<UIEvent> {
        None
    }
}