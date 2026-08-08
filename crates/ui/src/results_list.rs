use gpui::*;
use misojisho_core::jp_to_english_dictionary::JpToEnglishWord;
use misojisho_core::part_of_speech::{PartOfSpeech, VerbType};

use crate::traits::styled_ext::StyledExt;

#[derive(Clone)]
pub(crate) struct ResultDisplay {
    word: String,
    kana_reading: Vec<String>,
    primary_meaning: String,
    meanings: Vec<String>,
    part_of_speech: Vec<PosTag>,
}

#[derive(Clone)]
pub(crate) struct PosTag {
    label: String,
    bg_color: Rgba,
}

impl PosTag {
    fn from_pos(pos: &PartOfSpeech) -> Self {
        let bg_color = match pos {
            PartOfSpeech::Verb(verb) => match verb {
                VerbType::Ichidan => rgb(0xca3cff),
                VerbType::Godan => rgb(0xB4A0E5),
                VerbType::Irregular => rgb(0xcaffd0),
                VerbType::Transitive => rgb(0xc9e4e7),
                VerbType::Intransitive => rgb(0x1e1014),
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

impl ResultDisplay {
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
    results: Vec<ResultDisplay>,
}

impl ResultsList {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { results: vec![] }
    }

    pub(crate) fn set_results(&mut self, results: Vec<ResultDisplay>, cx: &mut Context<Self>) {
        self.results = results;
        cx.notify();
    }
}

impl Render for ResultsList {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let results = self.results.clone();

        uniform_list(
            "results",
            results.len(),
            move |visible_range, _window, _cx| {
                visible_range
                    .map(|i| {
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
                                                    .text_ellipsis(),
                                            )
                                            .child(div().child(results[i].word.clone()).text_3xl())
                                            .child(div().child(div().h_flex().gap_1().children(
                                                results[i].part_of_speech.iter().map(|tag| {
                                                    div()
                                                        .child(tag.label.clone())
                                                        .px_2()
                                                        .py_0p5()
                                                        .rounded_full()
                                                        .bg(tag.bg_color)
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
                                                    .font_weight(FontWeight::BOLD),
                                            )
                                            .child(
                                                div()
                                                    .child(results[i].meanings.join(" "))
                                                    .text_ellipsis(),
                                            ),
                                    ),
                            )
                            .text_color(rgb(0xFFFFFF))
                    })
                    .collect()
            },
        )
        .size_full()
    }
}
