use std::rc::Rc;

use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, rgb,
};

use crate::{
    results_list::{PosTag, ResultsList, WordEntry},
    traits::styled_ext::StyledExt,
    utils::title_bar_height,
};

pub struct WordDetailView {
    word_detail: Entity<WordDetail>,
    back_button_pressed: Option<Rc<dyn Fn(ResultsList, &mut Window, &mut Context<Self>)>>,
}

impl WordDetailView {
    pub fn new(word_entry: WordEntry, cx: &mut Context<Self>) -> Self {
        Self {
            word_detail: cx.new(|cx| WordDetail::new(word_entry, cx)),
            back_button_pressed: None,
        }
    }

    pub fn back_button_pressed(mut self) {
        // self.back_button_pressed = Some(Rc::new(callback))
        println!("Hit back")
    }
}

/// Wrapper for WordDetail
impl Render for WordDetailView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .child(div().h(title_bar_height(window)).w_full().bg(rgb(0xB9BBC6)))
            .text_color(rgb(0xfffff))
            .child(div().child("back").text_color(rgb(0xfffff)).px_4())
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
