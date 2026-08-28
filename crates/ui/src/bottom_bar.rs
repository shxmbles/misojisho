use std::rc::Rc;

use gpui::{
    Context, InteractiveElement, ParentElement, Render, StatefulInteractiveElement, Styled, Window,
    div, px, rgb,
};

use crate::{traits::styled_ext::StyledExt, utils::title_bar_height};

/// The bottom `side bar/panel navigation` bar which only toggles the side bar for now
pub struct BottomBar {
    on_toggle_sidebar: Option<Rc<dyn Fn(&mut Window, &mut Context<Self>)>>,
}

impl BottomBar {
    pub fn new() -> Self {
        Self {
            on_toggle_sidebar: None,
        }
    }

    pub fn on_toggle_sidebar(
        mut self,
        callback: impl Fn(&mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_toggle_sidebar = Some(Rc::new(callback));
        self
    }
}

impl Render for BottomBar {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let entity = cx.entity();
        div()
            .id("bottom-bar")
            .w_full()
            .h(title_bar_height(window))
            .h_flex()
            .items_center()
            .bg(rgb(0xB9BBC6))
            .child(
                div()
                    .id("sidebar-toggle")
                    .child("<")
                    .w(px(28.))
                    .px_4()
                    .on_click(move |_event, window, cx| {
                        entity.update(cx, |view, cx| {
                            if let Some(on_toggle) = view.on_toggle_sidebar.clone() {
                                on_toggle(window, cx);
                            }
                        });
                    }),
            )
    }
}
