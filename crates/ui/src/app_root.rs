use gpui::{
    AnyElement, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    div, rgb,
};
use misojisho_core::jp_to_english_dictionary::JpToEnglishDictionary;

use crate::{
    bottom_bar::BottomBar, results_list::WordEntry, search_page::SearchResultsView,
    sidebar::Sidebar, traits::styled_ext::StyledExt, utils::title_bar_height,
    word_detail::WordDetailView,
};

pub struct AppRoot {
    search_results: Entity<SearchResultsView>,
    current: Screen,
    sidebar: Entity<Sidebar>,
    bottom_bar: Entity<BottomBar>,
}

impl AppRoot {
    pub fn new(dictionary: JpToEnglishDictionary, cx: &mut Context<Self>) -> Self {
        let app_root = cx.entity();

        let bottom_bar = cx.new(|_cx| {
            let app_root = app_root.clone();
            BottomBar::new().on_toggle_sidebar(move |_window, cx| {
                app_root.update(cx, |app_root, cx| app_root.toggle_sidebar(cx))
            })
        });

        let sidebar = cx.new(|_cx| Sidebar::new());

        let search_results = cx.new(|cx| {
            SearchResultsView::new(dictionary, cx, move |word_entry, _window, cx| {
                let word_entry = word_entry.clone();
                app_root.update(cx, |app_root, cx| {
                    app_root.handle_result_click(word_entry, cx);
                });
            })
        });

        Self {
            search_results,
            current: Screen::SearchResults,
            sidebar,
            bottom_bar,
        }
    }

    fn handle_result_click(&mut self, context: WordEntry, cx: &mut Context<Self>) {
        let app_root = cx.entity();
        let word_detail = cx.new(|cx| {
            WordDetailView::new(context, cx).on_back_button_pressed(move |_context, _window, cx| {
                app_root.update(cx, |app_root, cx| {
                    app_root.handle_back_button_click(cx);
                })
            })
        });

        self.current = Screen::WordDetail(word_detail);
        cx.notify();
    }

    fn handle_back_button_click(&mut self, cx: &mut Context<Self>) {
        self.current = Screen::SearchResults;
        cx.notify();
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar.update(cx, |sidebar, cx| {
            sidebar.toggle(cx);
        });
    }
}

impl Render for AppRoot {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let screen: AnyElement = match &self.current {
            Screen::SearchResults => div()
                .v_flex()
                .size_full()
                .bg(rgb(0xFCFCFD))
                .child(div().h(title_bar_height(window)).w_full().bg(rgb(0xB9BBC6)))
                .child(div().flex_1().child(self.search_results.clone()))
                .into_any_element(),
            Screen::WordDetail(word_detail) => div()
                .v_flex()
                .size_full()
                .child(word_detail.clone())
                .into_any_element(),
        };

        div()
            .v_flex()
            .size_full()
            .child(
                div()
                    .h_flex()
                    .flex_1()
                    .child(self.sidebar.clone())
                    .child(div().flex_1().h_full().child(screen)),
            )
            .child(self.bottom_bar.clone())
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
    use crate::results_list::WordEntry;

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

    fn word_entry() -> WordEntry {
        WordEntry {
            main_kanji: "眠い".to_string(),
            kana_reading: vec!["ねむい".to_string()],
            primary_meaning: "sleepy".to_string(),
            meanings: vec!["drowsy".to_string()],
            part_of_speech: vec![],
        }
    }

    #[gpui::test]
    fn should_show_word_detail_screen_when_result_clicked(cx: &mut TestAppContext) {
        let (app_root, cx) = cx.add_window_view(|_window, cx| AppRoot::new(dictionary(), cx));

        app_root.update(cx, |view, cx| {
            view.handle_result_click(word_entry(), cx);
        });

        app_root.read_with(cx, |view, _cx| {
            assert!(matches!(view.current, Screen::WordDetail(_)));
        });
    }

    #[gpui::test]
    fn should_show_results_screen_when_back_button_clicked(cx: &mut TestAppContext) {
        let (app_root, cx) = cx.add_window_view(|_window, cx| AppRoot::new(dictionary(), cx));

        // 1. Click into detail view
        app_root.update(cx, |view, cx| {
            view.handle_result_click(word_entry(), cx);
        });

        // 2. Click the back button
        app_root.update(cx, |view, cx| {
            view.handle_back_button_click(cx);
        });

        app_root.read_with(cx, |view, _cx| {
            assert_eq!(Screen::SearchResults, view.current);
        });
    }
}
