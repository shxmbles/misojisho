use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, rgb,
};
use misojisho_core::jp_to_english_dictionary::JpToEnglishDictionary;

use crate::{
    search_page::SearchResultsView, traits::styled_ext::StyledExt, utils::title_bar_height,
    word_detail::WordDetailView,
};

pub struct AppRoot {
    search_results: Entity<SearchResultsView>,
    current: Screen,
}

impl AppRoot {
    pub fn new(dictionary: JpToEnglishDictionary, cx: &mut Context<Self>) -> Self {
        let app_root = cx.entity();

        let search_results = cx.new(|cx| {
            SearchResultsView::new(dictionary, cx, move |word_detail_context, _window, cx| {
                let props = word_detail_context.clone();
                let app_root_for_back = app_root.clone();
                let app_root_for_forward = app_root.clone();
                let word_detail = cx.new(|cx| {
                    WordDetailView::new(props.clone(), cx).on_back_button_pressed(
                        move |_context, _window, cx| {
                            app_root_for_back.update(cx, |app_root, cx| {
                                app_root.current = Screen::SearchResults;
                                cx.notify()
                            })
                        },
                    )
                });

                app_root_for_forward.update(cx, |app_root, cx| {
                    app_root.current = Screen::WordDetail(word_detail);
                    cx.notify();
                })
            })
        });

        Self {
            search_results: search_results,
            current: Screen::SearchResults,
        }
    }
}

impl Render for AppRoot {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match &self.current {
            Screen::SearchResults => {
                return div()
                    .v_flex()
                    .size_full()
                    .bg(rgb(0xFCFCFD))
                    // Reserved space for the (now-transparent) native titlebar's
                    // traffic-light buttons. Empty on purpose, not padding on
                    // real content.
                    .child(div().h(title_bar_height(window)).w_full().bg(rgb(0xB9BBC6)))
                    .child(div().flex_1().child(self.search_results.clone()));
            }
            Screen::WordDetail(word_detail) => {
                return div().v_flex().size_full().child(word_detail.clone());
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    SearchResults,
    WordDetail(Entity<WordDetailView>),
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;
    use misojisho_core::jp_to_english_dictionary::{JpToEnglishDictionary, JpToEnglishWord};
    use misojisho_core::part_of_speech::{PartOfSpeech, VerbType};

    use crate::app_root::{AppRoot, Screen};

    fn dictionary() -> JpToEnglishDictionary {
        JpToEnglishDictionary {
            words: vec![
                JpToEnglishWord {
                    id: "1000".to_string(),
                    kanji: Some(vec!["眠い".to_string()]),
                    kana_reading: vec!["ねむい".to_string()],
                    use_frequency: None,
                    english_meaning: vec!["sleepy".to_string(), "drowsy".to_string()],
                    part_of_speech: vec![PartOfSpeech::Adjective],
                    verb_conjugations: None,
                },
                JpToEnglishWord {
                    id: "1001".to_string(),
                    kanji: Some(vec!["行く".to_string()]),
                    kana_reading: vec!["いく".to_string()],
                    use_frequency: None,
                    english_meaning: vec!["to go".to_string()],
                    part_of_speech: vec![PartOfSpeech::Verb(VerbType::Godan)],
                    verb_conjugations: None,
                },
                JpToEnglishWord {
                    id: "1002".to_string(),
                    kanji: Some(vec!["猫".to_string()]),
                    kana_reading: vec!["ねこ".to_string()],
                    use_frequency: None,
                    english_meaning: vec!["cat".to_string()],
                    part_of_speech: vec![PartOfSpeech::Noun],
                    verb_conjugations: None,
                },
            ],
        }
    }

    #[gpui::test]
    fn should_show_results_screen(cx: &mut TestAppContext) {
        let (app_root, cx) = cx.add_window_view(|_window, cx| AppRoot::new(dictionary(), cx));

        app_root.read_with(cx, |view, _cx| {
            assert_eq!(Screen::SearchResults, view.current);
        });
    }
}
