use gpui::*;
use misojisho_core::jp_to_english_dictionary::JpToEnglishDictionary;

mod results_list;
mod search_page;
mod text_input;
mod traits;

use search_page::SearchPage;

pub fn run(dictionary: JpToEnglishDictionary) {
    Application::new().run(move |cx: &mut App| {
        text_input::bind_keys(cx);

        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::new("misojisho?")),
                appears_transparent: true,
                ..Default::default()
            }),
            ..Default::default()
        };

        let window = cx.open_window(window_options, move |
            _, cx| {
            cx.new(|cx| SearchPage::new(dictionary, cx))
        });

        if let Err(e) = window {
            eprintln!("failed to open window: {e}");
            cx.quit();
        }
    });
}
