use std::rc::Rc;

use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, rgb,
};

use crate::{
    results_list::{PosTag, WordEntry},
    text::SelectableText,
    traits::styled_ext::StyledExt,
    utils::title_bar_height,
};

pub struct WordDetailView {
    word_entry: WordEntry,
    word_detail: Entity<WordDetail>,
    on_back_button_pressed: Option<Rc<dyn Fn(WordEntry, &mut Window, &mut Context<Self>)>>,
}

impl WordDetailView {
    pub fn new(word_entry: WordEntry, cx: &mut Context<Self>) -> Self {
        Self {
            word_detail: cx.new(|cx| WordDetail::new(word_entry.clone(), cx)),
            word_entry,
            on_back_button_pressed: None,
        }
    }

    pub fn on_back_button_pressed(
        mut self,
        callback: impl Fn(WordEntry, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_back_button_pressed = Some(Rc::new(callback));
        self
    }
}

/// Wrapper for WordDetail
impl Render for WordDetailView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let word_entry = self.word_entry.clone();
        div()
            .size_full()
            .v_flex()
            .child(div().h(title_bar_height(window)).w_full().bg(rgb(0xB9BBC6)))
            .text_color(rgb(0xfffff))
            .child(
                div()
                    .id("back-button")
                    .child("back")
                    .text_color(rgb(0xfffff))
                    .px_4()
                    .on_click(move |_event, window, cx| {
                        entity.update(cx, |view, cx| {
                            if let Some(on_back) = view.on_back_button_pressed.clone() {
                                on_back(word_entry.clone(), window, cx);
                            }
                        });
                    }),
            )
            .child(div().size_full().flex_1().child(self.word_detail.clone()))
            .bg(rgb(0xFCFCFD))
    }
}

pub struct WordDetail {
    main_kanji: String,
    kana_reading: Vec<String>,
    primary_meaning: Entity<SelectableText>,
    meanings: Entity<SelectableText>,
    part_of_speech: Vec<PosTag>,
}

impl WordDetail {
    pub fn new(word_entry: WordEntry, cx: &mut Context<Self>) -> Self {
        Self {
            main_kanji: word_entry.main_kanji,
            kana_reading: word_entry.kana_reading,
            primary_meaning: cx.new(|cx| SelectableText::new(word_entry.primary_meaning, cx)),
            meanings: cx.new(|cx| SelectableText::new(word_entry.meanings.join(", "), cx)),
            part_of_speech: word_entry.part_of_speech,
        }
    }
}

/// Selected Words detail
impl Render for WordDetail {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex_1()
            .child(
                div()
                    .centered()
                    .v_flex()
                    .child(
                        div()
                            .v_flex()
                            .child(self.kana_reading.join(" | "))
                            .child(self.main_kanji.clone())
                            .text_color(rgb(0x646464)),
                    )
                    .child(
                        div()
                            .v_flex()
                            .child(div().child(self.primary_meaning.clone()).bg(rgb(0xff0000)))
                            .child(self.meanings.clone())
                            .text_color(rgb(0x1C2024)),
                    )
                    .child(
                        div()
                            .h_flex()
                            .gap_4()
                            .children(self.part_of_speech.iter().map(|p| {
                                div()
                                    .child(p.label.clone())
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(p.bg_color)
                            })),
                    ),
            )
            .text_color(rgb(0xffffff))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::TestAppContext;

    use super::*;

    #[gpui::test]
    fn should_map_word_entry_to_word_detail(cx: &mut TestAppContext) {
        let word_entry = WordEntry {
            main_kanji: "根気".to_string(),
            kana_reading: vec!["こんき".to_string()],
            primary_meaning: "patience".to_string(),
            meanings: vec!["perseverance".to_string()],
            part_of_speech: vec![PosTag {
                label: "Noun".to_string(),
                bg_color: rgb(0xffffff),
            }],
        };

        let (word_detail, cx) = cx.add_window_view(|_window, cx| WordDetail::new(word_entry, cx));

        word_detail.read_with(cx, |word_detail, cx| {
            assert_eq!(word_detail.main_kanji, "根気");
            assert_eq!(word_detail.kana_reading, vec!["こんき".to_string()]);
            assert_eq!(word_detail.primary_meaning.read(cx).content, "patience");
            assert_eq!(word_detail.meanings.read(cx).content, "perseverance");
            assert_eq!(word_detail.part_of_speech.len(), 1);
            assert_eq!(word_detail.part_of_speech[0].label, "Noun");
        });
    }

    #[gpui::test]
    fn should_store_word_detail(cx: &mut TestAppContext) {
        let word_entry = WordEntry {
            main_kanji: "笑う".to_string(),
            kana_reading: vec!["わらう".to_string()],
            primary_meaning: "to laugh".to_string(),
            meanings: vec!["to giggle".to_string()],
            part_of_speech: vec![PosTag {
                label: "Godan Verb".to_string(),
                bg_color: rgb(0xffffff),
            }],
        };

        let (word_detail, cx) =
            cx.add_window_view(|_window, cx| WordDetailView::new(word_entry, cx));

        word_detail.read_with(cx, |word_detail_view, _cx| {
            assert_eq!(word_detail_view.word_entry.main_kanji, "笑う");
            assert_eq!(
                word_detail_view.word_entry.kana_reading,
                vec!["わらう".to_string()]
            );
            assert_eq!(word_detail_view.word_entry.primary_meaning, "to laugh");
            assert_eq!(
                word_detail_view.word_entry.meanings,
                vec!["to giggle".to_string()]
            );
            assert_eq!(word_detail_view.word_entry.part_of_speech.len(), 1);
            assert_eq!(
                word_detail_view.word_entry.part_of_speech[0].label,
                "Godan Verb"
            );
        });
    }

    #[gpui::test]
    fn should_navigate_back(cx: &mut TestAppContext) {
        let word_entry = WordEntry {
            main_kanji: "眠い".to_string(),
            kana_reading: vec!["ねむい".to_string()],
            primary_meaning: "sleepy".to_string(),
            meanings: vec!["drowsy".to_string()],
            part_of_speech: vec![PosTag {
                label: "Adjective".to_string(),
                bg_color: rgb(0xffffff),
            }],
        };

        // Records whatever the callback is invoked with, so the test can
        // assert on it after triggering the callback below.
        let received: Rc<RefCell<Option<WordEntry>>> = Rc::new(RefCell::new(None));
        let received_for_callback = received.clone();

        let (word_detail, cx) = cx.add_window_view(|_window, cx| {
            WordDetailView::new(word_entry, cx).on_back_button_pressed(
                move |word_entry, _window, _cx| {
                    *received_for_callback.borrow_mut() = Some(word_entry);
                },
            )
        });

        // Copies the pointer to the closure
        // If no one called on_back the closure would return none
        word_detail.update_in(cx, |view, window, cx| {
            if let Some(on_back) = view.on_back_button_pressed.clone() {
                on_back(view.word_entry.clone(), window, cx);
            }
        });

        let received = received.borrow();
        let received = received
            .as_ref()
            .expect("on_back_button_pressed should have fired");
        assert_eq!(received.main_kanji, "眠い");
        assert_eq!(received.kana_reading, vec!["ねむい".to_string()]);
        assert_eq!(received.primary_meaning, "sleepy");
        assert_eq!(received.meanings, vec!["drowsy".to_string()]);
        assert_eq!(received.part_of_speech.len(), 1);
        assert_eq!(received.part_of_speech[0].label, "Adjective");
    }
}
