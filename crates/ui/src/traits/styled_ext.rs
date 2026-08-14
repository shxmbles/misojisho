use gpui::Styled;

pub trait StyledExt: Styled + Sized {
    fn v_flex(self) -> Self {
        self.flex().flex_col()
    }

    fn h_flex(self) -> Self {
        self.flex().flex_row()
    }

    fn centered(self) -> Self {
        self.items_center().justify_center()
    }
}

impl<T: Styled> StyledExt for T {}
