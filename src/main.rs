fn main() {
    match misojisho_core::parser::Parser::parse_xml("JMdict_e") {
        Ok(dictionary) => misojisho_ui::run(dictionary),
        Err(e) => eprintln!("{e}"),
    }
}
