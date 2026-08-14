use gpui::*;
use misojisho_core::jp_to_english_dictionary::JpToEnglishDictionary;

use crate::results_list::{ResultDisplay, ResultsList};
use crate::text_input::TextInput;
use crate::traits::styled_ext::StyledExt;

pub struct SearchPage {
    search_input: Entity<TextInput>,
    results_list: Entity<ResultsList>,
}

impl SearchPage {
    pub fn new(dictionary: JpToEnglishDictionary, cx: &mut Context<Self>) -> Self {
        let results_list = cx.new(|cx| ResultsList::new(cx));

        let results_list_for_submit = results_list.clone();

        let search_input = cx.new(move |cx| {
            TextInput::new("Search...", cx).on_submit(move |query, _window, cx| {
                let results = dictionary
                    .search(query)
                    .unwrap_or_default()
                    .into_iter()
                    .map(ResultDisplay::from_word)
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

impl Render for SearchPage {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // scales with the user's base font size (accessibility), with a 34px floor on
        // macOS, so traffic-light buttons never get covered even if
        // someone bumps up their font size.
        let title_bar_height = (1.75 * window.rem_size()).max(px(34.));

        div()
            .v_flex()
            .size_full()
            .bg(rgb(0xFCFCFD))
            // Reserved space for the (now-transparent) native titlebar's
            // traffic-light buttons. Empty on purpose, not padding on
            // real content.
            .child(div().h(title_bar_height).w_full().bg(rgb(0xB9BBC6)))
            .child(
                div()
                    .v_flex()
                    .flex_1()
                    .gap_2()
                    .p_4()
                    .child(
                        div()
                            .child(self.search_input.clone())
                            .border_2()
                            .border_color(rgb(0xD9D9E0)),
                    )
                    .child(self.results_list.clone()),
            )
    }
}
