use std::rc::Rc;

use gpui::*;
use misojisho_core::jp_to_english_dictionary::JpToEnglishWord;
use misojisho_core::part_of_speech::{PartOfSpeech, VerbType};

use crate::traits::styled_ext::StyledExt;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WordEntry {
    pub main_kanji: String,
    pub kana_reading: Vec<String>,
    pub primary_meaning: String,
    pub meanings: Vec<String>,
    pub part_of_speech: Vec<PosTag>,
}

#[derive(Clone, Debug, PartialEq)]
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
    // None only if kanji and kana_reading are both empty,
    // which shouldn't happen with real data
    // but keeps a malformed entry from showing up blank instead of being excluded.
    pub(crate) fn from_word(word: &JpToEnglishWord) -> Option<Self> {
        let display_word = word
            .kanji
            .as_ref()
            .and_then(|kanji| kanji.first())
            .or_else(|| word.kana_reading.first())
            .cloned()?;

        let part_of_speech = word.part_of_speech.iter().map(PosTag::from_pos).collect();

        let mut meanings = word.english_meaning.clone();
        let primary_meaning = if meanings.is_empty() {
            String::new()
        } else {
            meanings.remove(0)
        };

        Some(Self {
            main_kanji: display_word,
            kana_reading: word.kana_reading.clone(),
            primary_meaning,
            meanings,
            part_of_speech,
        })
    }
}

pub struct ResultsList {
    results: Vec<WordEntry>,
    scroll_handle: UniformListScrollHandle,
    on_result_click: Option<Rc<dyn Fn(&WordEntry, &mut Window, &mut Context<Self>)>>,
}

impl ResultsList {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            results: vec![],
            scroll_handle: UniformListScrollHandle::default(),
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
                                                    .child(results[i].main_kanji.clone())
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
        .track_scroll(self.scroll_handle.clone())
        .size_full()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{TestAppContext, rgb};
    use misojisho_core::jp_to_english_dictionary::JpToEnglishWord;
    use misojisho_core::part_of_speech::{PartOfSpeech, VerbType};

    use crate::results_list::{PosTag, ResultsList, WordEntry};

    fn word_with_multiple_variants() -> JpToEnglishWord {
        JpToEnglishWord {
            id: "1001".to_string(),
            kanji: Some(vec!["行く".to_string(), "往く".to_string()]),
            kana_reading: vec!["いく".to_string(), "ゆく".to_string()],
            use_frequency: None,
            english_meaning: vec![
                "to go".to_string(),
                "to move".to_string(),
                "to proceed".to_string(),
            ],
            part_of_speech: vec![PartOfSpeech::Verb(VerbType::Godan)],
            verb_conjugations: None,
        }
    }

    #[test]
    fn should_parse_from_word() {
        let word = JpToEnglishWord {
            id: "1000".to_string(),
            kanji: Some(vec!["眠い".to_string()]),
            kana_reading: vec!["ねむい".to_string()],
            use_frequency: None,
            english_meaning: vec!["sleepy".to_string(), "drowsy".to_string()],
            part_of_speech: vec![PartOfSpeech::Adjective],
            verb_conjugations: None,
        };

        let result =
            WordEntry::from_word(&word).expect("word has a kanji, so this should be Some");

        assert_eq!("眠い", result.main_kanji);
        assert_eq!(vec!["ねむい"], result.kana_reading);
        assert_eq!("sleepy", result.primary_meaning);
        assert_eq!(vec!["drowsy"], result.meanings);
        assert_eq!(
            vec![PosTag {
                label: "Adjective".to_string(),
                bg_color: rgb(0x84A98C)
            }],
            result.part_of_speech
        )
    }

    #[test]
    fn should_parse_main_kanji_when_multiple() {
        let word = WordEntry::from_word(&word_with_multiple_variants())
            .expect("word has a kanji, so this should be Some");
        assert_eq!("行く", word.main_kanji)
    }

    #[test]
    fn should_parse_multiple_kana_readings() {
        let word = WordEntry::from_word(&word_with_multiple_variants())
            .expect("word has a kanji, so this should be Some");
        assert_eq!(
            vec!["いく".to_string(), "ゆく".to_string()],
            word.kana_reading
        )
    }

    #[test]
    fn should_parse_primary_english_meaning_when_multiple() {
        let word = WordEntry::from_word(&word_with_multiple_variants())
            .expect("word has a kanji, so this should be Some");
        assert_eq!("to go", word.primary_meaning)
    }

    #[test]
    fn should_parse_main_kanji_to_kana_when_no_kanji() {
        let word = WordEntry::from_word(&JpToEnglishWord {
            kanji: None,
            kana_reading: vec!["ないものねだり".to_string()],
            ..Default::default()
        })
        .expect("word has a kana reading, so this should be Some");

        assert_eq!("ないものねだり", word.main_kanji);
    }

    #[test]
    fn should_be_excluded_from_list_when_kanji_and_kana_are_none() {
        let word = WordEntry::from_word(&JpToEnglishWord {
            ..Default::default()
        });

        assert_eq!(None, word)
    }

    #[test]
    fn should_map_pos_to_correct_label_and_color() {
        assert_eq!(
            PosTag {
                label: "Noun".to_string(),
                bg_color: rgb(0x6B9AC4)
            },
            PosTag::from_pos(&PartOfSpeech::Noun)
        );
        assert_eq!(
            PosTag {
                label: "Adjective".to_string(),
                bg_color: rgb(0x84A98C)
            },
            PosTag::from_pos(&PartOfSpeech::Adjective)
        );
        assert_eq!(
            PosTag {
                label: "Adverb".to_string(),
                bg_color: rgb(0xE9C46A)
            },
            PosTag::from_pos(&PartOfSpeech::Adverbs)
        );
        assert_eq!(
            PosTag {
                label: "Godan Verb".to_string(),
                bg_color: rgb(0xB4A0E5)
            },
            PosTag::from_pos(&PartOfSpeech::Verb(VerbType::Godan))
        );
        assert_eq!(
            PosTag {
                label: "Ichidan Verb".to_string(),
                bg_color: rgb(0xca3cff)
            },
            PosTag::from_pos(&PartOfSpeech::Verb(VerbType::Ichidan))
        );
    }

    #[gpui::test]
    fn should_register_on_click(cx: &mut TestAppContext) {
        let word =
            WordEntry::from_word(&word_with_multiple_variants()).expect("This should be some");

        let received: Rc<RefCell<Option<WordEntry>>> = Rc::new(RefCell::new(None));
        let received_for_callback = received.clone();

        let (results_list, cx) = cx.add_window_view(|_window, cx| {
            ResultsList::new(cx).on_result_click(move |word_entry, _window, _cx| {
                *received_for_callback.borrow_mut() = Some(word_entry.clone());
            })
        });

        results_list.update_in(cx, |view, window, cx| {
            if let Some(on_result_click) = view.on_result_click.clone() {
                on_result_click(&word, window, cx);
            }
        });

        let received = received.borrow();
        let received = received
            .as_ref()
            .expect("on_result_click should have fired");
        assert_eq!(received, &word);
    }
}
