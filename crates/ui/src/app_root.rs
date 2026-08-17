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
            SearchResultsView::new(dictionary, cx, move |word_entry, _window, cx| {
                let word_entry = word_entry.clone();
                let word_detail = cx.new(|cx| WordDetailView::new(word_entry, cx));

                app_root.update(cx, |app_root, cx| {
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

pub enum Screen {
    SearchResults,
    WordDetail(Entity<WordDetailView>),
}
