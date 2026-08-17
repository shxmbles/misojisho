use gpui::*;
use misojisho_core::jp_to_english_dictionary::JpToEnglishDictionary;

use crate::results_list::{ResultsList, WordEntry};
use crate::text_input::TextInput;
use crate::traits::styled_ext::StyledExt;

pub struct SearchResultsView {
    search_input: Entity<TextInput>,
    results_list: Entity<ResultsList>,
}

impl SearchResultsView {
    pub fn new(
        dictionary: JpToEnglishDictionary,
        cx: &mut Context<Self>,
        on_result_click: impl Fn(&WordEntry, &mut Window, &mut Context<ResultsList>) + 'static,
    ) -> Self {
        let results_list = cx.new(|cx| ResultsList::new(cx).on_result_click(on_result_click));

        let results_list_for_submit = results_list.clone();

        let search_input = cx.new(move |cx| {
            TextInput::new("Search...", cx).on_submit(move |query, _window, cx| {
                let results = dictionary
                    .search(query)
                    .unwrap_or_default()
                    .into_iter()
                    .map(WordEntry::from_word)
                    .collect();

                results_list_for_submit.update(cx, |results_list, cx| {
                    results_list.set_results(results, cx);
                });
            })
        });

        Self {
            search_input,
            results_list,
        }
    }
}

impl Render for SearchResultsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .gap_2()
            .p_4()
            .child(
                div()
                    .child(self.search_input.clone())
                    .border_2()
                    .border_color(rgb(0xD9D9E0)),
            )
            .child(div().flex_1().child(self.results_list.clone()))
    }
}
