use gpui::*;
use misojisho_core::jp_to_english_dictionary::JpToEnglishDictionary;
use text_input::TextInput;

mod text_input;
mod traits;

pub fn run(dictionary: JpToEnglishDictionary) {
    Application::new().run(move |cx: &mut App| {
        text_input::bind_keys(cx);

        let window = cx.open_window(WindowOptions::default(), move |window, cx| {
            window.set_window_title("misojisho?");
            cx.new(|cx| {
                TextInput::new("Search...", cx).on_submit(move |query, _window, _cx| {
                    match dictionary.search(query) {
                        Some(results) => {
                            println!("{} results found", results.len());
                            let (top, rest) = results.split_at(results.len().min(3));
                            println!("{top:#?}");
                            if !rest.is_empty() {
                                println!("...and {} more", rest.len());
                            }
                        }
                        None => println!("No results found for \"{query}\""),
                    }
                })
            })
        });

        if let Err(e) = window {
            eprintln!("failed to open window: {e}");
            cx.quit();
        }
    });
}
