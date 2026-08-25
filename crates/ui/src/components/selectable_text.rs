use gpui::{
    App, Context, FocusHandle, Focusable, HighlightStyle, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Render, SharedString,
    StyledText, TextLayout, Window, div, rgba,
};

use crate::selection::Selection;

#[derive(Clone, Copy, PartialEq)]
enum Charkind {
    Word,
    Whitespace,
    Punctuation,
}

fn char_kind(c: char) -> Charkind {
    if c.is_alphanumeric() {
        Charkind::Word
    } else if c.is_whitespace() {
        Charkind::Whitespace
    } else {
        Charkind::Punctuation
    }
}

/// Text that can be highlighted, copied, right clicked to open copy menu (coming soon).
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
    fn double_click(&mut self, offset: usize, cx: &mut Context<Self>) {
        let content: &str = &self.content;

        let Some(kind) = content[offset..]
            .chars()
            .next()
            .or_else(|| content[offset..].chars().next_back())
            .map(char_kind)
        else {
            return;
        };

        let start = content[..offset]
            .char_indices()
            .rev()
            .take_while(|&(_, c)| char_kind(c) == kind)
            .last()
            .map_or(offset, |(i, _)| i);

        let end = content[offset..]
            .char_indices()
            .take_while(|&(_, c)| char_kind(c) == kind)
            .last()
            .map_or(offset, |(i, c)| offset + i + c.len_utf8());

        self.selection.move_to(start);
        self.selection.select_to(end);
        cx.notify()
    }

    /// Selects the entire 'sentence'
    fn triple_click(&mut self, cx: &mut Context<Self>) {
        let start = 0;
        let end = self.content.len();
        self.selection.move_to(start);
        self.selection.select_to(end);
        cx.notify();
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
            self.extend_selection(offset, cx);
        }
    }

    fn extend_selection(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selection.select_to(offset);
        cx.notify();
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle);
        let offset = self
            .text_layout
            .index_for_position(event.position)
            .unwrap_or_else(|i| i);
        self.handle_click(offset, event.click_count, cx);
    }

    fn handle_click(&mut self, offset: usize, click_count: usize, cx: &mut Context<Self>) {
        match click_count {
            2 => self.double_click(offset, cx),
            3 => self.triple_click(cx),
            _ => {
                self.selection.start_selecting();
                self.selection.move_to(offset);
                cx.notify();
            }
        }
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
    use gpui::{
        Entity, Modifiers, MouseButton, MouseDownEvent, Point, SharedString, TestAppContext,
        TextLayout, VisualTestContext,
    };

    use crate::{components::SelectableText, selection::Selection};

    fn make_selectable_text<'a>(
        cx: &'a mut TestAppContext,
        content: &str,
        initial_cursor: usize,
    ) -> (Entity<SelectableText>, &'a mut VisualTestContext) {
        let content = content.to_string();
        cx.add_window_view(move |_window, cx| SelectableText {
            content: SharedString::new(content),
            selection: Selection::new(initial_cursor..initial_cursor),
            focus_handle: cx.focus_handle(),
            text_layout: TextLayout::default(),
        })
    }

    // Double click

    #[gpui::test]
    fn should_highlight_whole_word_on_double_click_from_start(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx)
        });
        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..5);
        })
    }

    #[gpui::test]
    fn should_highlight_whole_word_on_double_click_from_middle_of_word(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 3);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..5);
        })
    }

    #[gpui::test]
    fn should_highlight_whole_word_on_double_click_from_end_of_word(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 4);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..5);
        })
    }

    #[gpui::test]
    fn should_select_only_punctuation_on_double_click(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hi, there", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..2);
        })
    }

    #[gpui::test]
    fn should_select_only_whitespace_on_double_click(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hi there", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..2);
        })
    }

    #[gpui::test]
    fn should_select_nothing_on_double_click_past_end_of_content(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 11);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 11..11);
        })
    }

    #[gpui::test]
    fn should_select_nothing_on_double_click_when_content_is_empty(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..0);
        })
    }

    #[gpui::test]
    fn should_select_whole_word_when_word_touches_end_of_content(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 8);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 6..11);
        })
    }

    #[gpui::test]
    fn should_select_only_boundary_space_on_double_click_between_two_words(
        cx: &mut TestAppContext,
    ) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello me", 5);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.double_click(text.selection.cursor_offset(), cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 5..6);
        })
    }

    // Triple click

    #[gpui::test]
    fn should_highlight_whole_sentence_on_triple_click(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "what am i doing?", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.triple_click(cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..16);
        })
    }

    #[gpui::test]
    fn should_highlight_whole_sentence_on_triple_click_kanji_kana(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "坂本さん、いますか。", 6);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.triple_click(cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..30);
        })
    }

    #[gpui::test]
    fn should_select_nothing_on_triple_click_when_content_is_empty(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.triple_click(cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..0);
        })
    }

    // Handle click

    #[gpui::test]
    fn should_start_selecting_and_move_cursor_on_single_click(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.handle_click(3, 1, cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 3..3);
            assert!(text.selection.is_selecting());
        })
    }

    #[gpui::test]
    fn should_select_whole_word_when_click_count_is_two(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.handle_click(3, 2, cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..5);
        })
    }

    #[gpui::test]
    fn should_select_whole_sentence_when_click_count_is_three(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "what am i doing?", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.handle_click(9, 3, cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..16);
        })
    }

    #[gpui::test]
    fn should_start_selecting_on_click_count_outside_one_through_three(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "what am i doing?", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.handle_click(9, 0, cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 9..9);
            assert!(text.selection.is_selecting());
        })
    }

    // Extend selection

    #[gpui::test]
    fn should_extend_selection_to_offset_on_mouse_move_when_selecting(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 0);

        selectable_text.update_in(cx, |text, _window, cx| {
            text.selection.start_selecting();
            text.extend_selection(6, cx);
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..6);
        })
    }

    // Click outside

    #[gpui::test]
    fn should_reset_selection_to_zero_on_click_outside(cx: &mut TestAppContext) {
        let (selectable_text, cx) = make_selectable_text(cx, "hello world", 0);

        selectable_text.update_in(cx, |text, window, cx| {
            text.selection.move_to(3);
            text.selection.select_to(7);

            text.on_click_outside(
                &MouseDownEvent {
                    button: MouseButton::Left,
                    position: Point::default(),
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    first_mouse: false,
                },
                window,
                cx,
            );
        });

        selectable_text.read_with(cx, |text, _| {
            assert_eq!(text.selection.selected_range, 0..0);
        })
    }
}
