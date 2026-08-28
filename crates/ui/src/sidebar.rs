use gpui::{Context, ParentElement, Render, Styled, div, prelude::FluentBuilder, px, rgb};

use crate::{traits::styled_ext::StyledExt, utils::title_bar_height};

/// The side navigation bar which will have saved words (coming soon)
pub struct Sidebar {
    open: bool,
}

impl Sidebar {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.open = !self.open;
        cx.notify();
    }
}

impl Render for Sidebar {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().h_flex().when(self.open, |this| {
            this.child(
                div()
                    .child(div().h(title_bar_height(window)).w_full().bg(rgb(0xB9BBC6)))
                    .w(px(180.))
                    .h_full()
                    .border_r_1()
                    .border_color(rgb(0xD9D9E0))
                    .bg(rgb(0xF3F3F5)),
            )
        })
    }
}
