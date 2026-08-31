use gpui::App;

mod button;
mod selectable_text;
mod text_input;

pub use button::Button;
pub use selectable_text::SelectableText;
pub use text_input::TextInput;

pub fn bind_keys(cx: &mut App) {
    selectable_text::bind_keys(cx);
    text_input::bind_keys(cx);
}
