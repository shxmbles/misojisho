use crate::{
    conjugations::VerbConjugations, part_of_speech::PartOfSpeech, use_frequency::UseFrequency,
};

const NO_PRIORITY: u8 = u8::MAX;

#[derive(Debug, PartialEq)]
pub struct JpToEnglishWord {
    /// JMdict entry sequence number (`<ent_seq>`) which is unique per entry.
    pub id: String,
    /// Kanji spellings (`<keb>`). Not every word has a kanji form.
    pub kanji: Option<Vec<String>>,
    /// Kana reading(s) (`<reb>`). Every entry has at least one.
    pub kana_reading: Vec<String>,
    /// Priority/frequency tags (`<ke_pri>`) which indicates common usage
    /// (e.g. "news1", "ichi1").
    pub use_frequency: Option<Vec<UseFrequency>>,
    /// English glosses (`<gloss>`).
    pub english_meaning: Vec<String>,
    /// Part-of-speech codes (`<pos>`), e.g. "&v5r;", "&vi;" - see JMdict's
    /// entity list in the DTD for what each code expands to.
    pub part_of_speech: Vec<PartOfSpeech>,
    /// Verb conjugations built from either Kanji reading(dictionary form)
    /// Or kana reading (dictionary form)
    pub verb_conjugations: Option<Vec<VerbConjugations>>,
}

#[derive(Default, Debug)]
pub struct JpToEnglishDictionary {
    pub words: Vec<JpToEnglishWord>,
}

impl JpToEnglishDictionary {
    /// Search through dictionary
    /// 1. Determines the kind of search: Japanese | English
    /// 2. Filters results containing the search query
    /// 3. Sorts by key: If contains frequency: Sort by frequency otherwise give lowest priority
    /// 4. Sorts by priority of use frequency
    /// 5. Sorts by english definition priority: If there is one
    /// 6. Sorts by Japanese dictionary priority: If there is one
    /// Returns: Vector of &JapaneseWord pertaining to the search query
    pub fn search(&self, query: &str) -> Option<Vec<&JpToEnglishWord>> {
        if query.is_empty() {
            return None;
        }

        let search_type = SearchType::from(query);
        let query_lower = query.to_lowercase();

        // 1. Filter for results containing what a user is searching for
        let mut results: Vec<&JpToEnglishWord> = self
            .words
            .iter()
            .filter(|value| match search_type {
                SearchType::English => value
                    .english_meaning
                    .iter()
                    .any(|m| m.contains(&query_lower)),
                SearchType::Japanese => {
                    value.kanji.iter().flatten().any(|k| k.contains(query))
                        || value.kana_reading.iter().any(|m| m.contains(query))
                }
            })
            .collect();

        if results.is_empty() {
            return None;
        }

        results.sort_by_key(|word| {
            let freqs = match word.use_frequency.as_ref() {
                Some(f) => f,
                None => return (NO_PRIORITY, NO_PRIORITY),
            };

            let primary = freqs
                .iter()
                .map(|f| f.priority())
                .min()
                .unwrap_or(NO_PRIORITY);
            let nf = freqs
                .iter()
                .find_map(|nf| match nf {
                    UseFrequency::NewsFrequency(n) => Some(*n),
                    _ => None,
                })
                .unwrap_or(NO_PRIORITY);

            (primary, nf)
        });

        match search_type {
            SearchType::English => {
                // 3. Sort by english definition priority
                results.sort_by_key(|word| word.english_definition_priority(query));

                Some(results)
            }
            SearchType::Japanese => {
                // 4. Sort by japanese dictionary form priority
                results.sort_by_key(|word| word.japanese_dictionary_form_priority(query));

                Some(results)
            }
        }
    }
}

impl JpToEnglishWord {
    /// Prioritize by:
    /// 1. First english definition: If the first english definition contains the search query it is prioritized
    /// 2. Then if the list of english meanings contains to search query
    /// 3. None: no priority
    pub fn english_definition_priority(&self, query: &str) -> Option<u8> {
        let first_containing_definition = self
            .english_meaning
            .first()
            .is_some_and(|d| d.contains(query));

        let partial_match = self
            .english_meaning
            .iter()
            .any(|definition| definition.contains(query));

        match (first_containing_definition, partial_match) {
            (true, _) => Some(0),
            (_, true) => Some(1),
            _ => None,
        }
    }

    /// Prioritize by:
    /// 1. Exact kanji search match
    /// 2. Then if a word contains the search query
    /// 3. None: no priority
    pub fn japanese_dictionary_form_priority(&self, query: &str) -> Option<u8> {
        let exact_match = self
            .kanji
            .as_deref()
            .and_then(|k| k.first())
            .is_some_and(|k| k == query);

        let partial_match = self.kanji.iter().flatten().any(|q| q.contains(query));

        match (exact_match, partial_match) {
            (true, _) => Some(0),
            (_, true) => Some(1),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum SearchType {
    English,
    Japanese,
}

impl From<&str> for SearchType {
    fn from(value: &str) -> Self {
        if value.chars().any(|c| matches!(c, '\u{3040}'..='\u{9FFF}')) {
            return SearchType::Japanese;
        }

        // Everything that isn't considered japanese is considered as english
        // as we do not support other languages as of yet.
        SearchType::English
    }
}

#[cfg(test)]
mod tests {
    use crate::use_frequency::UseFrequency;

    use super::*;

    /// `part_of_speech` and `verb_conjugations` don't affect `search()`'s
    /// filtering or sorting at all, so they're defaulted to empty/`None`
    /// here rather than exposed as parameters.
    fn make_word(
        id: &str,
        kanji: Option<Vec<&str>>,
        kana_reading: Vec<&str>,
        use_frequency: Option<Vec<UseFrequency>>,
        english_meaning: Vec<&str>,
    ) -> JpToEnglishWord {
        JpToEnglishWord {
            id: id.to_string(),
            kanji: kanji.map(|k| k.into_iter().map(String::from).collect()),
            kana_reading: kana_reading.into_iter().map(String::from).collect(),
            use_frequency,
            english_meaning: english_meaning.into_iter().map(String::from).collect(),
            part_of_speech: Vec::new(),
            verb_conjugations: None,
        }
    }

    fn make_dictionary(words: Vec<JpToEnglishWord>) -> JpToEnglishDictionary {
        JpToEnglishDictionary { words }
    }

    #[test]
    fn should_map_to_english_search_type() {
        let actual = SearchType::from("Alexis Sanchez");

        assert_eq!(actual, SearchType::English)
    }

    #[test]
    fn should_map_to_japanese_search_type() {
        let actual = SearchType::from("海賊王になる男だ");

        assert_eq!(actual, SearchType::Japanese)
    }

    #[test]
    fn should_return_top_result_when_kanji_exactly_matches_query() {
        let dictionary = make_dictionary(vec![
            make_word(
                "1358280",
                Some(vec!["食べる"]),
                vec!["たべる"],
                Some(vec![UseFrequency::Ichi(1), UseFrequency::News(1)]),
                vec!["to eat"],
            ),
            make_word(
                "1591050",
                Some(vec!["話す"]),
                vec!["はなす"],
                Some(vec![UseFrequency::Ichi(1)]),
                vec!["to speak", "to talk"],
            ),
            make_word(
                "1157170",
                None,
                vec!["する"],
                Some(vec![UseFrequency::Ichi(1)]),
                vec!["to do"],
            ),
            make_word(
                "1587040",
                Some(vec!["今日"]),
                vec!["きょう", "こんにち"],
                Some(vec![UseFrequency::Ichi(1), UseFrequency::News(1)]),
                vec!["today", "this day"],
            ),
            make_word(
                "1591090",
                Some(vec!["大きい"]),
                vec!["おおきい"],
                Some(vec![
                    UseFrequency::Ichi(1),
                    UseFrequency::News(1),
                    UseFrequency::LoanWord(1),
                ]),
                vec!["big", "large"],
            ),
        ]);

        let actual = dictionary
            .search("食べる")
            .and_then(|v| v.into_iter().next())
            .expect("expected a search result for 食べる");

        assert_eq!(actual.id, "1358280");
    }

    #[test]
    fn should_return_none_when_query_is_empty() {
        let dictionary = make_dictionary(vec![make_word(
            "1358280",
            Some(vec!["食べる"]),
            vec!["たべる"],
            None,
            vec!["to eat"],
        )]);

        let actual = dictionary.search("");

        assert_eq!(actual, None);
    }

    #[test]
    fn should_return_none_when_no_words_match_query() {
        let dictionary = make_dictionary(vec![make_word(
            "1358280",
            Some(vec!["食べる"]),
            vec!["たべる"],
            None,
            vec!["to eat"],
        )]);

        let actual = dictionary.search("寝る");

        assert_eq!(actual, None);
    }

    #[test]
    fn should_return_result_when_english_meaning_matches_query() {
        let dictionary = make_dictionary(vec![
            make_word(
                "1358280",
                Some(vec!["食べる"]),
                vec!["たべる"],
                None,
                vec!["to eat"],
            ),
            make_word(
                "1591090",
                Some(vec!["大きい"]),
                vec!["おおきい"],
                None,
                vec!["big", "large"],
            ),
        ]);

        let actual = dictionary
            .search("big")
            .and_then(|v| v.into_iter().next())
            .expect("expected a search result for 'big'");

        assert_eq!(actual.id, "1591090");
    }

    #[test]
    fn should_match_partial_kanji_query() {
        let dictionary = make_dictionary(vec![make_word(
            "1358280",
            Some(vec!["食べる"]),
            vec!["たべる"],
            None,
            vec!["to eat"],
        )]);

        // "食べ" is a prefix of "食べる", not an exact match.
        let actual = dictionary
            .search("食べ")
            .and_then(|v| v.into_iter().next())
            .expect("expected a partial kanji match for 食べ");

        assert_eq!(actual.id, "1358280");
    }

    #[test]
    fn should_prioritize_higher_frequency_word_when_definition_priority_ties() {
        // Neither word's first english meaning contains "run", so both tie
        // at definition priority Some(1) via their second meaning. The
        // frequency-based sort (which runs before the definition-priority
        // sort, and is stable) should still put the higher-frequency word first.
        let dictionary = make_dictionary(vec![
            make_word(
                "slow_freq",
                None,
                vec!["あるく"],
                Some(vec![UseFrequency::News(2)]),
                vec!["to walk", "to run"],
            ),
            make_word(
                "fast_freq",
                None,
                vec!["じょぐ"],
                Some(vec![UseFrequency::Ichi(1)]),
                vec!["to jog", "to run"],
            ),
        ]);

        let actual = dictionary
            .search("run")
            .and_then(|v| v.into_iter().next())
            .expect("expected a search result for 'run'");

        assert_eq!(actual.id, "fast_freq");
    }

    #[test]
    fn should_sort_word_with_no_frequency_last_when_definition_priority_ties() {
        // Both words have the same kanji, so both tie at japanese_dictionary_form_priority
        // Some(0) (exact match). With that tie, the frequency-based sort (which runs
        // first and is stable) should decide the order: the word with a real frequency
        // should come before the word with no frequency data (NO_PRIORITY fallback).
        let dictionary = make_dictionary(vec![
            make_word(
                "no_freq",
                Some(vec!["飲む"]),
                vec!["のむ"],
                None,
                vec!["to drink"],
            ),
            make_word(
                "has_freq",
                Some(vec!["飲む"]),
                vec!["のむ"],
                Some(vec![UseFrequency::Ichi(1)]),
                vec!["to drink"],
            ),
        ]);

        let actual = dictionary
            .search("飲む")
            .expect("expected search results for 飲む");

        let ids: Vec<&str> = actual.iter().map(|w| w.id.as_str()).collect();

        assert_eq!(ids, vec!["has_freq", "no_freq"]);
    }
}
