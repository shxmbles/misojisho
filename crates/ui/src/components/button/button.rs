use gpui::{App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Window};

use crate::{components::button::ButtonLike, traits::clickable::Clickable};

#[derive(IntoElement)]
pub struct Button {
    base: ButtonLike,
    label: SharedString,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            base: ButtonLike::new(id),
            label: label.into(),
        }
    }
}

impl Clickable for Button {
    fn on_click(
        mut self,
        handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.base = self.base.on_click(handler);
        self
    }

    fn cursor_style(mut self, cursor_style: gpui::CursorStyle) -> Self {
        self.base = self.base.cursor_style(cursor_style);
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        self.base.child(self.label)
    }
}
