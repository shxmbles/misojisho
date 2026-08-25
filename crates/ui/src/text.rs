use gpui::{
    App, Context, FocusHandle, Focusable, HighlightStyle, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Render, SharedString,
    StyledText, TextLayout, Window, div, rgba,
};

use crate::selection::Selection;

// The data
pub struct SelectableText {
    pub content: SharedString,
    selection: Selection,
    focus_handle: FocusHandle,
    text_layout: TextLayout,
}

impl Focusable for SelectableText {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl SelectableText {
    pub fn new(content: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            content: content.into(),
            selection: Selection::new(0..0),
            focus_handle: cx.focus_handle(),
            text_layout: TextLayout::default(),
        }
    }

    /// Selects the entire word
    fn double_click(&mut self, cx: &mut Context<Self>) {
        // The problem im running into here is that
        // I need to select to up until there is a " " or a "" and i think
        // this would need to go both ways
        self.selection.select_to(self.content.len());
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.selection.stop_selecting();
        cx.notify();
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selection.is_selecting() {
            let offset = self
                .text_layout
                .index_for_position(event.position)
                .unwrap_or_else(|i| i);
            self.selection.select_to(offset);
            cx.notify();
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        println!("hit");
        // let offset = self.index_for_mouse_position(event.position, window.line_height());

        // match event.click_count {
        //     1 => {
        //         println!("tapped once")
        //     }
        //     2 => {
        //         println!("tapped twice")
        //     }
        //     3 => {
        //         println!("tapped thrice")
        //     }
        //     // NO OP
        //     _ => {}
        window.focus(&self.focus_handle);
        let offset = self
            .text_layout
            .index_for_position(event.position)
            .unwrap_or_else(|i| i);
        self.selection.start_selecting();
        self.selection.move_to(offset);
        cx.notify();
    }

    fn on_click_outside(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.blur();
        self.selection.move_to(0);
        cx.notify();
    }
}

impl Render for SelectableText {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text_style = window.text_style();

        let styled_text = StyledText::new(self.content.clone()).with_default_highlights(
            &text_style,
            [(
                self.selection.selected_range.clone(),
                HighlightStyle {
                    background_color: Some(rgba(0x3b82f640).into()),
                    ..Default::default()
                },
            )],
        );

        self.text_layout = styled_text.layout().clone();

        div()
            .id("selectable-text")
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_down_out(cx.listener(Self::on_click_outside))
            .child(styled_text)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn should_be_something() {}
}
