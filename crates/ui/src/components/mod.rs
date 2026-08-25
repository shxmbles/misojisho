use gpui::App;

mod selectable_text;

pub use selectable_text::SelectableText;

pub fn bind_keys(cx: &mut App) {
    selectable_text::bind_keys(cx);
}
