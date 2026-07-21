use std::io;

use misojisho::parser::Parser;

fn main() -> io::Result<()> {
    let result = Parser::parse_xml("JMdict_e");
    let mut search_term = String::new();

    match result {
        Ok(jp_to_english) => {
            println!("Search for a word:");
            io::stdin().read_line(&mut search_term)?;
            let search_term = search_term.trim();
            let results = jp_to_english.search(search_term).unwrap_or_default();
            println!("{} results found", results.len());

            let (top, rest) = results.split_at(results.len().min(3));
            println!("Results {:#?}", top);

            if !rest.is_empty() {
                println!("Press Enter to see the remaining {} results...", rest.len());
                let mut pause = String::new();
                io::stdin().read_line(&mut pause)?;
                println!("Results {:#?}", rest);
            }
        }
        Err(e) => eprintln!("{e}"),
    }

    Ok(())
}
