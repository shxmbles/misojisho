use std::{ops::Range, rc::Rc};

use gpui::*;

use crate::{selection::Selection, text::surrounding_word_range::surrounding_word_range};

// The actions we will use with KeyBindings
actions!(
    text_input,
    [
        Backspace,
        Left,
        Right,
        SecondaryLeft,
        SecondaryRight,
        SelectAll,
        CopyText,
        Cut,
        Paste,
        SecondaryShiftLeft,
        SecondaryShiftRight,
        SecondaryBackspace,
        Esc,
        Undo,
        Enter
    ]
);

/// A single-line, GPUI-native text input field with full IME support
/// (Japanese/CJK composition), mouse-driven selection, standard keyboard
/// shortcuts, and undo.
///
/// Construct with [`TextInput::new`], optionally chaining
/// [`TextInput::on_submit`] to react to Enter:
///
/// ```ignore
/// cx.new(|cx| {
///     TextInput::new("Search...", cx).on_submit(|query, window, cx| {
///         // ...
///     })
/// })
/// ```
pub struct TextInput {
    content: SharedString,
    placeholder: SharedString,

    // Either where the cursor is or what we have highlighted
    selection: Selection,

    // Only relevant during IME if japanese characters are still in a draft we underline them
    marked_range: Option<Range<usize>>,
    focus_handle: FocusHandle,
    on_submit: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut Context<Self>)>>,

    // The text we recently deleted (the whole word rather than just 'h', 'e' 'y')
    undo_stack: Vec<(SharedString, Range<usize>)>,
    last_edit_end: Option<usize>,
    last_edit_was_insertion: Option<bool>,

    // For mouse clicks + drags
    // and IME when the Kanji pops up:
    // To position it in the right place
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
}

impl TextInput {
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        let selection = Selection::new(0..0);

        Self {
            content: "".into(),
            placeholder: placeholder.into(),
            selection,
            marked_range: None,
            focus_handle: cx.focus_handle().tab_stop(true),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        }
    }

    // Builder-style: cx.new(|cx| TextInput::new("Search...", cx).on_submit(...))
    pub fn on_submit(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(callback));
        self
    }

    /// Where the cursor moves to. Clears any selection.
    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selection.move_to(offset);
        self.last_edit_end = None;
        self.last_edit_was_insertion = None;
        cx.notify();
    }

    /// Moves just the selection's head (not the anchor), so it grows or
    /// shrinks. Flips direction if dragged past the anchor.
    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selection.select_to(offset);
        self.last_edit_end = None;
        self.last_edit_was_insertion = None;
        cx.notify();
    }

    /// Turns a mouse position into a text offset. Falls back to the start
    /// or end if the click is outside the field, or 0 if nothing's been
    /// painted yet.
    fn index_for_mouse_position(&self, position: gpui::Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }

    /// When the mouse is pressed down. Focuses the field, starts a
    /// selection, and moves the cursor to the click.
    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle(cx));
        let offset = self.index_for_mouse_position(event.position);
        self.handle_click(offset, event.click_count, cx);
    }

    /// When the left mouse button is released, stops selecting.
    fn on_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.selection.stop_selecting();
    }

    /// Clicking outside the text input unfocuses it.
    fn on_click_outside(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        window.blur();
    }

    /// While dragging, updates the selection to follow the mouse.
    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selection.is_selecting() {
            let offset = self.index_for_mouse_position(event.position);
            self.select_to(offset, cx);
        }
    }

    fn double_click(&mut self, offset: usize, cx: &mut Context<Self>) {
        let range = surrounding_word_range(&self.content, offset);
        self.selection.move_to(range.start);
        self.selection.select_to(range.end);
        cx.notify();
    }

    /// Selects the entire 'sentence'
    fn triple_click(&mut self, cx: &mut Context<Self>) {
        self.selection.move_to(0);
        self.selection.select_to(self.content.len());
        cx.notify();
    }

    // Everything EntityInputHandler sends/expects is in UTF-16 code units
    // (the OS side speaks UTF-16). `content` is a normal Rust String/str
    // internally, which is UTF-8. These convert between the two.
    //
    // Free functions taking an explicit `text: &str`, not just methods on
    // `self.content` because new_selected_range (in
    // replace_and_mark_text_in_range) is relative to new_text specifically,
    // a different string than self.content.
    fn offset_to_utf16(text: &str, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in text.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn offset_from_utf16(text: &str, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        Self::offset_to_utf16(&self.content, range.start)
            ..Self::offset_to_utf16(&self.content, range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        Self::offset_from_utf16(&self.content, range_utf16.start)
            ..Self::offset_from_utf16(&self.content, range_utf16.end)
    }

    // Byte offset of the character boundary just before/after `offset`.
    // Using char boundaries (not raw byte + 1) so this doesn't try to
    // land in the middle of a multi-byte character like 走.
    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .char_indices()
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .char_indices()
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    // Keyboard shortcuts
    // "secondary" resolves to cmd on macOS, ctrl on Windows/Linux

    fn left(&mut self, _: &Left, _window: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.previous_boundary(self.selection.cursor_offset());
        self.move_to(cursor, cx);
    }

    fn right(&mut self, _: &Right, _window: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.next_boundary(self.selection.cursor_offset());
        self.move_to(cursor, cx);
    }

    fn secondary_shift_left(
        &mut self,
        _: &SecondaryShiftLeft,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(0, cx);
    }

    fn secondary_shift_right(
        &mut self,
        _: &SecondaryShiftRight,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.content.len(), cx);
    }

    fn secondary_backspace(
        &mut self,
        _: &SecondaryBackspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let cursor = self.selection.cursor_offset();
        self.replace_text_in_range(Some(0..cursor), "", window, cx);
    }

    fn secondary_left(&mut self, _: &SecondaryLeft, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn secondary_right(
        &mut self,
        _: &SecondaryRight,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_to(self.content.len(), cx);
    }

    fn esc(&mut self, _: &Esc, window: &mut Window, _cx: &mut Context<Self>) {
        window.blur();
    }

    fn undo(&mut self, _: &Undo, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some((content, selected_range)) = self.undo_stack.pop() {
            self.content = content;
            self.selection.selected_range = selected_range;
            self.selection.selection_reversed = false;
            self.marked_range = None;
            self.last_edit_end = None;
            self.last_edit_was_insertion = None;
            cx.notify();
        }
    }

    fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_submit) = self.on_submit.clone() {
            let content = self.content.clone();
            on_submit(&content, window, cx);
        }
    }

    fn select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.selection.move_to(0);
        self.selection.select_to(self.content.len());
        cx.notify();
    }

    fn copy_text(&mut self, _: &CopyText, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.selection.selected_range.is_empty() {
            let selected_content = self.content[self.selection.selected_range.clone()].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(selected_content));
        }
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        let Some(copied_string) = cx.read_from_clipboard().and_then(|i| i.text()) else {
            return;
        };

        let sanitized = copied_string.replace(['\n', '\r'], " ");
        self.replace_text_in_range(None, &sanitized, window, cx);
        cx.notify();
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

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selection.selected_range.is_empty() {
            let selected_content = self.content[self.selection.selected_range.clone()].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(selected_content));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selection.selected_range.is_empty() {
            let cursor = self.selection.cursor_offset();
            let start = self.previous_boundary(cursor);
            self.selection.selected_range = start..cursor;
        }
        self.replace_text_in_range(None, "", window, cx);
    }
}

impl EntityInputHandler for TextInput {
    /// What text is in this range? Reports back the exact range it used,
    /// since the OS's range may need adjusting to a char boundary.
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        adjusted_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    /// Selected text range in UTF-16
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selection.selected_range),
            reversed: self.selection.selection_reversed,
        })
    }

    /// Is there an IME composition in progress right now, and where.
    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range.as_ref().map(|r| self.range_to_utf16(r))
    }

    /// Composition finished. Clears the marked range.
    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    /// Replaces the text in selected range
    /// Places cursor after last edit
    /// Checks if last edit was insertion.
    /// Marked range to nil
    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .map(|r| self.range_from_utf16(&r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selection.selected_range.clone());

        let is_insertion = !text.is_empty();
        let continues_previous_edit = match self.last_edit_was_insertion {
            Some(true) => is_insertion && self.last_edit_end == Some(range.start),
            Some(false) => !is_insertion && self.last_edit_end == Some(range.end),
            None => false,
        };
        if !continues_previous_edit {
            self.undo_stack
                .push((self.content.clone(), self.selection.selected_range.clone()));
        }

        self.content =
            (self.content[..range.start].to_owned() + text + &self.content[range.end..]).into();

        let cursor = range.start + text.len();
        self.selection.selected_range = cursor..cursor;
        self.last_edit_end = Some(cursor);
        self.last_edit_was_insertion = Some(is_insertion);
        self.marked_range = None;
        cx.notify();
    }

    /// Updates the underlined IME preview text while composing.
    /// new_selected_range is relative to new_text, not self.content.
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range
            .map(|r| self.range_from_utf16(&r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selection.selected_range.clone());

        self.content =
            (self.content[..range.start].to_owned() + new_text + &self.content[range.end..]).into();

        // The new preview text is now the marked (bracketed) range.
        self.marked_range = if new_text.is_empty() {
            None
        } else {
            Some(range.start..range.start + new_text.len())
        };

        // new_selected_range is UTF-16 units relative to new_text
        // specifically (not self.content), so convert against new_text.
        self.selection.selected_range = new_selected_range
            .map(|r| {
                let start = Self::offset_from_utf16(new_text, r.start);
                let end = Self::offset_from_utf16(new_text, r.end);
                range.start + start..range.start + end
            })
            .unwrap_or_else(|| {
                let cursor = range.start + new_text.len();
                cursor..cursor
            });

        cx.notify();
    }

    /// Where does this range sit on screen. Used to position the IME
    /// candidate popup right under the text being composed.
    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        let line = self.last_layout.as_ref()?;
        Some(Bounds::from_corners(
            point(bounds.left() + line.x_for_index(range.start), bounds.top()),
            point(bounds.left() + line.x_for_index(range.end), bounds.bottom()),
        ))
    }

    /// What character is at this screen position. Converts the answer
    /// back to UTF-16 before returning it.
    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line = self.last_layout.as_ref()?;
        let index = line.index_for_x(point.x - bounds.left())?;
        Some(Self::offset_to_utf16(&self.content, index))
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// A custom element: the only way to get manual control over shaping text
// and painting a cursor/underline ourselves, instead of GPUI laying out a
// plain string for us.
struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let cursor = input.selection.cursor_offset();
        let selected_range = input.selection.selected_range.clone();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), hsla(0., 0., 0., 0.3))
        } else {
            (content, style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let (selection, cursor) = if selected_range.is_empty() {
            let cursor_pos = line.x_for_index(cursor);
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(2.), bounds.bottom() - bounds.top()),
                    ),
                    rgb(0x3b82f6),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    rgba(0x3b82f640),
                )),
                None,
            )
        };

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );

        // Selection highlight paints first, underneath the text.
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }

        // prepaint() should always produce a line for this same frame. If it
        // somehow didn't, there's nothing to paint text with, but we can
        // still draw the cursor and keep the field usable instead of
        // crashing the whole app over one bad frame.
        let Some(line) = prepaint.line.take() else {
            eprintln!("no shaped line to paint this frame");
            if focus_handle.is_focused(window) {
                if let Some(cursor) = prepaint.cursor.take() {
                    window.paint_quad(cursor);
                }
            }
            self.input.update(cx, |input, _cx| {
                input.last_bounds = Some(bounds);
            });
            return;
        };

        // Painting can fail, but not worth crashing the app over.
        if let Err(e) = line.paint(bounds.origin, window.line_height(), window, cx) {
            eprintln!("failed to paint text line: {e}");
        }

        if focus_handle.is_focused(window) {
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("TextInput")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::secondary_left))
            .on_action(cx.listener(Self::secondary_shift_left))
            .on_action(cx.listener(Self::secondary_shift_right))
            .on_action(cx.listener(Self::secondary_backspace))
            .on_action(cx.listener(Self::secondary_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::copy_text))
            .on_action(cx.listener(Self::esc))
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_down_out(cx.listener(Self::on_click_outside))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .w(px(300.))
            .p_2()
            .rounded(px(8.))
            .bg(rgb(0xffffff))
            .text_color(rgb(0x000000))
            .child(TextElement { input: cx.entity() })
    }
}

// Registers this component's keyboard shortcuts. Called once from run().
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, None),
        KeyBinding::new("left", Left, None),
        KeyBinding::new("right", Right, None),
        KeyBinding::new("secondary-left", SecondaryLeft, None),
        KeyBinding::new("secondary-right", SecondaryRight, None),
        KeyBinding::new("secondary-a", SelectAll, None),
        KeyBinding::new("secondary-c", CopyText, None),
        KeyBinding::new("secondary-x", Cut, None),
        KeyBinding::new("secondary-shift-left", SecondaryShiftLeft, None),
        KeyBinding::new("secondary-shift-right", SecondaryShiftRight, None),
        KeyBinding::new("secondary-backspace", SecondaryBackspace, None),
        KeyBinding::new("escape", Esc, None),
        KeyBinding::new("secondary-z", Undo, None),
        KeyBinding::new("enter", Enter, None),
        KeyBinding::new("secondary-v", Paste, None),
    ]);
}

#[cfg(test)]
mod tests {
    use gpui::{ClipboardItem, EntityInputHandler, SharedString, TestAppContext};

    use crate::selection::Selection;

    use super::{
        Backspace, CopyText, Cut, Esc, Left, Paste, Right, SecondaryBackspace, SecondaryLeft,
        SecondaryRight, SecondaryShiftLeft, SecondaryShiftRight, SelectAll, TextInput, Undo,
    };

    // Backspace

    #[gpui::test]
    fn should_delete_previous_plain_ascii_char_on_backspace(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(5..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.backspace(&Backspace, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "hell");
        });
    }

    #[gpui::test]
    fn should_delete_previous_mutli_byte_char_on_backspace(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "海賊になる男だ".into(),
            placeholder: "".into(),
            selection: Selection::new(21..21),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.backspace(&Backspace, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "海賊になる男");
        });
    }

    #[gpui::test]
    fn should_delete_nothing_and_not_go_out_of_bounds(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.backspace(&Backspace, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "");
        });
    }

    // `Left` <-
    #[gpui::test]
    fn should_navigate_one_char_left_on_plain_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(5..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.left(&Left, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 4..4);
        });
    }

    #[gpui::test]
    fn should_navigate_one_char_left_on_multi_byte_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "子宮から一年早まった人間は偉いっすね".into(),
            placeholder: "".into(),
            selection: Selection::new(54..54),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.left(&Left, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 51..51);
        });
    }

    #[gpui::test]
    fn should_navigate_no_where_and_not_out_of_bounds(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.left(&Left, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0);
        });
    }

    // `Right` ->

    #[gpui::test]
    fn should_navigate_one_char_right_on_plain_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(4..4),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.right(&Right, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 5..5);
        });
    }

    #[gpui::test]
    fn should_navigate_one_char_right_on_multi_byte_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "安心安心".into(),
            placeholder: "".into(),
            selection: Selection::new(9..9),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.right(&Right, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 12..12);
        });
    }

    #[gpui::test]
    fn should_navigate_no_where_and_not_out_of_bounds_for_right(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(5..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.right(&Right, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 5..5);
        });
    }

    // `secondary left`  ⌘ - <-
    #[gpui::test]
    fn should_navigate_to_start_from_end(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(5..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_left(&SecondaryLeft, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0)
        })
    }

    #[gpui::test]
    fn should_navigate_to_start_from_end_with_multi_byte_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "俺は誰".into(),
            placeholder: "".into(),
            selection: Selection::new(9..9),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_left(&SecondaryLeft, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0)
        })
    }

    #[gpui::test]
    fn should_navigate_to_start(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(3..3),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_left(&SecondaryLeft, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0)
        })
    }

    #[gpui::test]
    fn should_navigate_to_start_with_multi_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "お腹がすいた".into(),
            placeholder: "".into(),
            selection: Selection::new(6..6),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_left(&SecondaryLeft, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0)
        })
    }

    #[gpui::test]
    fn should_navigate_no_where_and_stay_in_bounds_when_empty(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_left(&SecondaryLeft, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0)
        })
    }

    // `secondary right`  ⌘ - ->

    #[gpui::test]
    fn should_navigate_to_end_from_start(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_right(&SecondaryRight, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 5..5)
        })
    }

    #[gpui::test]
    fn should_navigate_to_end_from_start_with_multi_byte_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "温泉に行きたいな".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_right(&SecondaryRight, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 24..24)
        })
    }

    #[gpui::test]
    fn should_navigate_to_end(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(2..2),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_right(&SecondaryRight, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 5..5)
        })
    }

    #[gpui::test]
    fn should_navigate_to_end_with_multi_byte_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "コンピューター".into(),
            placeholder: "".into(),
            selection: Selection::new(8..8),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_right(&SecondaryRight, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 21..21)
        })
    }

    #[gpui::test]
    fn should_navigate_to_end_when_empty_and_stay_in_bounds(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_right(&SecondaryRight, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0)
        })
    }

    // `secondary-a` select all

    #[gpui::test]
    fn should_select_all_text(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(2..2),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.select_all(&SelectAll, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..5);
        });
    }

    #[gpui::test]
    fn should_select_all_multi_byte_text(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "こんにちは".into(),
            placeholder: "".into(),
            selection: Selection::new(3..3),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.select_all(&SelectAll, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..15);
        });
    }

    #[gpui::test]
    fn should_select_all_when_empty_and_stay_in_bounds(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.select_all(&SelectAll, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0);
        });
    }

    // `secondary-c` copy

    #[gpui::test]
    fn should_copy_selected_text_to_clipboard(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello world".into(),
            placeholder: "".into(),
            selection: Selection::new(0..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.copy_text(&CopyText, window, cx);
        });

        text_input.read_with(cx, |text_input, cx| {
            let clipboard_text = cx.read_from_clipboard().and_then(|item| item.text());
            assert_eq!(clipboard_text, Some("hello".to_string()));
            assert_eq!(text_input.content.to_string(), "hello world");
        });
    }

    // `secondary-v` paste

    #[gpui::test]
    fn should_insert_clipboard_text_on_paste(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string("hello".to_string()));
            text_input.paste(&Paste, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "hello");
        });
    }

    #[gpui::test]
    fn should_replace_selected_text_on_paste(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello world".into(),
            placeholder: "".into(),
            selection: Selection::new(0..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string("goodbye".to_string()));
            text_input.paste(&Paste, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "goodbye world");
        });
    }

    #[gpui::test]
    fn should_replace_newlines_with_spaces_on_paste(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string("hello\nworld".to_string()));
            text_input.paste(&Paste, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "hello world");
        });
    }

    #[gpui::test]
    fn should_do_nothing_on_paste_when_clipboard_has_no_text(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(5..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.paste(&Paste, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "hello");
        });
    }

    #[gpui::test]
    fn should_copy_selected_multi_byte_text_to_clipboard(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "海賊王におれはなる".into(),
            placeholder: "".into(),
            selection: Selection::new(0..9),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.copy_text(&CopyText, window, cx);
        });

        text_input.read_with(cx, |_text_input, cx| {
            let clipboard_text = cx.read_from_clipboard().and_then(|item| item.text());
            assert_eq!(clipboard_text, Some("海賊王".to_string()));
        });
    }

    #[gpui::test]
    fn should_not_touch_clipboard_when_nothing_selected(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(3..3),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |_text_input, _window, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string("sentinel".to_string()));
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.copy_text(&CopyText, window, cx);
        });

        text_input.read_with(cx, |_text_input, cx| {
            let clipboard_text = cx.read_from_clipboard().and_then(|item| item.text());
            assert_eq!(clipboard_text, Some("sentinel".to_string()));
        });
    }

    // `secondary-x` cut

    #[gpui::test]
    fn should_cut_selected_text(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello world".into(),
            placeholder: "".into(),
            selection: Selection::new(0..6),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.cut(&Cut, window, cx);
        });

        text_input.read_with(cx, |text_input, cx| {
            let clipboard_text = cx.read_from_clipboard().and_then(|item| item.text());
            assert_eq!(clipboard_text, Some("hello ".to_string()));
            assert_eq!(text_input.content.to_string(), "world");
            assert_eq!(text_input.selection.selected_range, 0..0);
        });
    }

    #[gpui::test]
    fn should_cut_selected_multi_byte_text(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "海賊王におれはなる".into(),
            placeholder: "".into(),
            selection: Selection::new(0..9),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.cut(&Cut, window, cx);
        });

        text_input.read_with(cx, |text_input, cx| {
            let clipboard_text = cx.read_from_clipboard().and_then(|item| item.text());
            assert_eq!(clipboard_text, Some("海賊王".to_string()));
            eprintln!("{}", text_input.content.to_string());
            assert_eq!(text_input.content.to_string(), "におれはなる");
        });
    }

    #[gpui::test]
    fn should_not_cut_anything_when_nothing_selected(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(3..3),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.cut(&Cut, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "hello");
        });
    }

    // `secondary-shift-left` select to start

    #[gpui::test]
    fn should_select_to_start_from_end(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(5..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_shift_left(&SecondaryShiftLeft, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..5);
            assert!(text_input.selection.selection_reversed);
        });
    }

    #[gpui::test]
    fn should_select_to_start_with_multi_byte_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "俺は誰".into(),
            placeholder: "".into(),
            selection: Selection::new(9..9),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_shift_left(&SecondaryShiftLeft, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..9);
            assert!(text_input.selection.selection_reversed);
        });
    }

    #[gpui::test]
    fn should_select_nothing_when_already_at_start(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_shift_left(&SecondaryShiftLeft, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..0);
        });
    }

    // `secondary-shift-right` select to end

    #[gpui::test]
    fn should_select_to_end_from_start(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_shift_right(&SecondaryShiftRight, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..5);
            assert!(!text_input.selection.selection_reversed);
        });
    }

    #[gpui::test]
    fn should_select_to_end_with_multi_byte_ascii(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "温泉に行きたいな".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_shift_right(&SecondaryShiftRight, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 0..24);
            assert!(!text_input.selection.selection_reversed);
        });
    }

    #[gpui::test]
    fn should_select_nothing_when_already_at_end(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(5..5),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_shift_right(&SecondaryShiftRight, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.selection.selected_range, 5..5);
        });
    }

    // `secondary-backspace` delete from start of text to cursor

    #[gpui::test]
    fn should_delete_from_start_to_cursor(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello world".into(),
            placeholder: "".into(),
            selection: Selection::new(6..6),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.secondary_backspace(&SecondaryBackspace, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "world");
            assert_eq!(text_input.selection.selected_range, 0..0);
        });
    }

    // `escape` blur focus

    #[gpui::test]
    fn should_blur_on_escape(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "hello".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        text_input.update_in(cx, |text_input, window, _cx| {
            window.focus(&text_input.focus_handle);
            assert!(text_input.focus_handle.is_focused(window));
        });

        text_input.update_in(cx, |text_input, window, cx| {
            text_input.esc(&Esc, window, cx);
            assert!(!text_input.focus_handle.is_focused(window));
        });
    }

    // Undo

    #[gpui::test]
    fn should_undo_whole_word(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![(SharedString::new("hello"), 0..5)],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        // Type "hello"
        text_input.update_in(cx, |text_input, window, cx| {
            text_input.replace_text_in_range(None, "hello", window, cx);
        });

        // Select it all and delete it
        text_input.update_in(cx, |text_input, window, cx| {
            text_input.select_all(&SelectAll, window, cx);
            text_input.replace_text_in_range(None, "", window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "");
        });

        // One undo should bring "hello" back. typing and deleting are
        // different edit kinds, so they're separate undo groups.
        text_input.update_in(cx, |text_input, window, cx| {
            text_input.undo(&Undo, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "hello");
        });
    }

    #[gpui::test]
    fn should_undo_whole_word_with_multi_byte_char(cx: &mut TestAppContext) {
        let (text_input, cx) = cx.add_window_view(|_window, cx| TextInput {
            content: "".into(),
            placeholder: "".into(),
            selection: Selection::new(0..0),
            marked_range: None,
            focus_handle: cx.focus_handle(),
            on_submit: None,
            last_layout: None,
            last_bounds: None,
            undo_stack: vec![],
            last_edit_end: None,
            last_edit_was_insertion: None,
        });

        // Type "いい天気っすね"
        text_input.update_in(cx, |text_input, window, cx| {
            text_input.replace_text_in_range(None, "いい天気っすね", window, cx);
        });

        // Select it all and delete it
        text_input.update_in(cx, |text_input, window, cx| {
            text_input.select_all(&SelectAll, window, cx);
            text_input.replace_text_in_range(None, "", window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "");
        });

        // One undo should bring "いい天気っすね" back. typing and deleting are
        // different edit kinds, so they're separate undo groups.
        text_input.update_in(cx, |text_input, window, cx| {
            text_input.undo(&Undo, window, cx);
        });

        text_input.read_with(cx, |text_input, _| {
            assert_eq!(text_input.content.to_string(), "いい天気っすね");
        });
    }
}
