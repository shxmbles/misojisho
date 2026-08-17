use gpui::{Pixels, Window, px};

pub fn title_bar_height(window: &Window) -> Pixels {
    (1.75 * window.rem_size()).max(px(34.))
}
