use std::rc::Rc;

use gpui::*;
use misojisho_core::jp_to_english_dictionary::JpToEnglishWord;
use misojisho_core::part_of_speech::{PartOfSpeech, VerbType};

use crate::traits::styled_ext::StyledExt;

#[derive(Clone)]
pub(crate) struct WordEntry {
    pub word: String,
    pub kana_reading: Vec<String>,
    pub primary_meaning: String,
    pub meanings: Vec<String>,
    pub part_of_speech: Vec<PosTag>,
}

#[derive(Clone)]
pub(crate) struct PosTag {
    pub label: String,
    pub bg_color: Rgba,
}

impl PosTag {
    fn from_pos(pos: &PartOfSpeech) -> Self {
        let bg_color = match pos {
            PartOfSpeech::Verb(verb) => match verb {
                VerbType::Ichidan => rgb(0xca3cff),
                VerbType::Godan => rgb(0xB4A0E5),
                VerbType::Irregular => rgb(0xcaffd0),
                VerbType::Transitive => rgb(0xc9e4e7),
                VerbType::Intransitive => rgb(0x80838D),
            },
            PartOfSpeech::Noun => rgb(0x6B9AC4),
            PartOfSpeech::Adjective => rgb(0x84A98C),
            PartOfSpeech::Adverbs => rgb(0xE9C46A),
        };

        Self {
            label: pos.to_string(),
            bg_color,
        }
    }
}

impl WordEntry {
    pub(crate) fn from_word(word: &JpToEnglishWord) -> Self {
        let display_word = word
            .kanji
            .as_ref()
            .and_then(|kanji| kanji.first())
            .or_else(|| word.kana_reading.first())
            .cloned()
            .unwrap_or_default();

        let part_of_speech = word.part_of_speech.iter().map(PosTag::from_pos).collect();

        let mut meanings = word.english_meaning.clone();
        let primary_meaning = if meanings.is_empty() {
            String::new()
        } else {
            meanings.remove(0)
        };

        Self {
            word: display_word,
            kana_reading: word.kana_reading.clone(),
            primary_meaning,
            meanings,
            part_of_speech,
        }
    }
}

pub struct ResultsList {
    results: Vec<WordEntry>,
    on_result_click: Option<Rc<dyn Fn(&WordEntry, &mut Window, &mut Context<Self>)>>,
}

impl ResultsList {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            results: vec![],
            on_result_click: None,
        }
    }

    pub(crate) fn set_results(&mut self, results: Vec<WordEntry>, cx: &mut Context<Self>) {
        self.results = results;
        cx.notify();
    }

    pub fn on_result_click(
        mut self,
        callback: impl Fn(&WordEntry, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_result_click = Some(Rc::new(callback));
        self
    }
}

impl Render for ResultsList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let results = self.results.clone();
        let entity = cx.entity();

        uniform_list(
            "results",
            results.len(),
            move |visible_range, _window, _cx| {
                visible_range
                    .map(|i| {
                        // Context for all the rows
                        let entity = entity.clone();
                        let word_entry = results[i].clone();
                        div()
                            .flex_1()
                            .py_4()
                            .child(
                                div()
                                    .h_flex()
                                    .child(
                                        div()
                                            .v_flex()
                                            .flex_1()
                                            .max_w(px(400.))
                                            .child(
                                                div()
                                                    .child(results[i].kana_reading.join(" | "))
                                                    .text_ellipsis()
                                                    .text_color(rgb(0x646464)),
                                            )
                                            .child(
                                                div()
                                                    .child(results[i].word.clone())
                                                    .text_3xl()
                                                    .text_color(rgb(0x1C2024)),
                                            )
                                            .child(div().child(div().h_flex().gap_1().children(
                                                results[i].part_of_speech.iter().map(|tag| {
                                                    div()
                                                        .child(tag.label.clone())
                                                        .px_2()
                                                        .py_0p5()
                                                        .rounded_full()
                                                        .bg(tag.bg_color)
                                                        .text_color(rgb(0x1C2024))
                                                }),
                                            ))),
                                    )
                                    .child(
                                        div()
                                            .v_flex()
                                            .flex_1()
                                            .max_w(px(600.))
                                            .child(
                                                div()
                                                    .child(results[i].primary_meaning.clone())
                                                    .text_lg()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0x1C2024)),
                                            )
                                            .child(
                                                div()
                                                    .child(results[i].meanings.join(" "))
                                                    .text_ellipsis()
                                                    .text_color(rgb(0x60646C)),
                                            ),
                                    ),
                            )
                            .id(("result", i))
                            .on_click(move |_event, window, cx| {
                                entity.update(cx, |result_list, cx| {
                                    if let Some(on_result_click) =
                                        result_list.on_result_click.clone()
                                    {
                                        on_result_click(&word_entry, window, cx)
                                    }
                                });
                            })
                    })
                    .collect()
            },
        )
        .size_full()
    }
}
