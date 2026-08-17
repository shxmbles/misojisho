use std::rc::Rc;

use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, rgb,
};

use crate::{
    results_list::{PosTag, WordEntry},
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
        return Self {
            word_detail: cx.new(|cx| WordDetail::new(word_entry.clone(), cx)),
            word_entry,
            on_back_button_pressed: None,
        };
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
            .child(div().size_full().child(self.word_detail.clone()))
            .bg(rgb(0xFCFCFD))
    }
}

#[derive(Default)]
pub struct WordDetail {
    main_kanji: String,
    kana_reading: Vec<String>,
    primary_meaning: String,
    meanings: Vec<String>,
    part_of_speech: Vec<PosTag>,
}

impl WordDetail {
    pub fn new(word_entry: WordEntry, _cx: &mut Context<Self>) -> Self {
        Self {
            main_kanji: word_entry.word,
            kana_reading: word_entry.kana_reading,
            primary_meaning: word_entry.primary_meaning,
            meanings: word_entry.meanings,
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
                            .child(self.primary_meaning.clone())
                            .child(self.meanings.join(", "))
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
