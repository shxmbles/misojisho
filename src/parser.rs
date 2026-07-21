use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    mem::take,
};

use crate::{
    conjugations::VerbConjugations,
    jp_to_english_dictionary::{JpToEnglishDictionary, JpToEnglishWord},
    part_of_speech::PartOfSpeech,
    use_frequency::UseFrequency,
};

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Parser {}

impl Parser {
    const ENTRY_START: &str = "<entry>";
    const ENTRY_END: &str = "</entry>";
    const ENTRY_SEQ_START: &str = "<ent_seq>";
    const ENTRY_SEQ_END: &str = "</ent_seq>";
    const KEB_START: &str = "<keb>";
    const KEB_END: &str = "</keb>";
    const REB_START: &str = "<reb>";
    const REB_END: &str = "</reb>";
    const KE_PRI_START: &str = "<ke_pri>";
    const KE_PRI_END: &str = "</ke_pri>";
    const GLOSS_START: &str = "<gloss>";
    const GLOSS_END: &str = "</gloss>";
    const POS_START: &str = "<pos>";
    const POS_END: &str = "</pos>";

    fn extract_all_tags<'a>(s: &'a str, open: &str, close: &str) -> Option<Vec<&'a str>> {
        let mut results = Vec::new();
        let mut rest = s;

        while let Some((_, outer)) = rest.split_once(open) {
            let Some((inner, outer_close)) = outer.split_once(close) else {
                break;
            };
            results.push(inner);
            rest = outer_close;
        }
        if results.is_empty() {
            return None;
        }

        Some(results)
    }

    /// Parse xml file into a Dictionary of `ParsedJapaneseWords`
    pub fn parse_xml(file: &str) -> Result<JpToEnglishDictionary, ParserError> {
        let file = File::open(file)?;
        let buf_reader = BufReader::new(file);
        let mut dict = JpToEnglishDictionary::default();
        let mut in_entry = false;
        let mut current_entry = String::new();
        let mut entries: Vec<String> = Vec::new();

        for line in buf_reader.lines() {
            let line = line?;
            match line.trim() {
                Self::ENTRY_START => {
                    in_entry = true;
                    current_entry.clear();
                }
                Self::ENTRY_END => {
                    in_entry = false;
                    entries.push(take(&mut current_entry));
                }
                _ if in_entry => current_entry.push_str(&line),
                _ => {}
            }
        }

        let mut seen_pos: HashSet<PartOfSpeech> = HashSet::new();

        for word in &entries {
            let entry_seq = word
                .split_once(Self::ENTRY_SEQ_START)
                .and_then(|(_, after)| after.split_once(Self::ENTRY_SEQ_END))
                .map(|(inner, _)| inner);
            let keb = Self::extract_all_tags(word, Self::KEB_START, Self::KEB_END);
            let reb = Self::extract_all_tags(word, Self::REB_START, Self::REB_END);
            let ke_pri = Self::extract_all_tags(word, Self::KE_PRI_START, Self::KE_PRI_END);
            let gloss = Self::extract_all_tags(word, Self::GLOSS_START, Self::GLOSS_END);
            let pos = Self::extract_all_tags(word, Self::POS_START, Self::POS_END);

            // If no entry_seq it's invalid.
            if let Some(entry_seq) = entry_seq {
                let kanji: Option<Vec<String>> =
                    keb.map(|k| k.into_iter().map(String::from).collect());

                let kana_reading: Vec<String> = reb
                    .map(|r| r.into_iter().map(String::from).collect())
                    .unwrap_or_default();

                let use_frequency: Option<Vec<UseFrequency>> = ke_pri.map(|v| {
                    v.into_iter()
                        .filter_map(UseFrequency::from_priority_code)
                        .collect()
                });

                let english_meaning = gloss
                    .map(|g| g.into_iter().map(String::from).collect())
                    .unwrap_or_default();

                let part_of_speech: Vec<PartOfSpeech> = pos
                    .map(|p| {
                        seen_pos.clear();
                        p.into_iter()
                            .filter_map(PartOfSpeech::from_pos_code)
                            .filter(|pos| seen_pos.insert(*pos))
                            .collect()
                    })
                    .unwrap_or_default();

                let dict_form: Option<String> = kanji
                    .as_ref()
                    .and_then(|k| k.first())
                    .or_else(|| kana_reading.first())
                    .cloned();

                let verb_conjugations = dict_form
                    .map(|form| {
                        part_of_speech
                            .iter()
                            .filter_map(|pos| match pos {
                                PartOfSpeech::Verb(verb_type) => {
                                    VerbConjugations::conjugate(*verb_type, &form)
                                }
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    })
                    .filter(|v| !v.is_empty());

                dict.words.push(JpToEnglishWord {
                    id: entry_seq.to_owned(),
                    kanji,
                    kana_reading,
                    use_frequency,
                    english_meaning,
                    part_of_speech,
                    verb_conjugations,
                });
            }
        }

        Ok(dict)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::part_of_speech::VerbType;

    use super::*;

    #[test]
    fn should_extract_all_keb() {
        let entry_with_multiple_keb: &str = "\
        <ent_seq>1610400</ent_seq>\
        <k_ele><keb>分かる</keb><ke_pri>ichi1</ke_pri><ke_pri>news1</ke_pri></k_ele>\
        <k_ele><keb>判る</keb></k_ele>\
        <k_ele><keb>解る</keb></k_ele>\
        <r_ele><reb>わかる</reb></r_ele>\
        <sense><pos>&v5r;</pos><pos>&vi;</pos><gloss>to understand</gloss><gloss>to comprehend</gloss></sense>\
        ";
        let actual =
            Parser::extract_all_tags(entry_with_multiple_keb, Parser::KEB_START, Parser::KEB_END);
        assert_eq!(actual, Some(vec!["分かる", "判る", "解る"]))
    }

    #[test]
    fn should_extract_all_reb() {
        let entry_with_multiple_reb: &str = "\
        <ent_seq>1587040</ent_seq>\
        <k_ele><keb>今日</keb><ke_pri>ichi1</ke_pri><ke_pri>news1</ke_pri></k_ele>\
        <r_ele><reb>きょう</reb></r_ele>\
        <r_ele><reb>こんにち</reb></r_ele>\
        <sense><pos>&n-adv;</pos><pos>&n-t;</pos><gloss>today</gloss><gloss>this day</gloss></sense>\
        ";
        let actual =
            Parser::extract_all_tags(entry_with_multiple_reb, Parser::REB_START, Parser::REB_END);
        assert_eq!(actual, Some(vec!["きょう", "こんにち"]));
    }

    #[test]
    fn should_extract_all_ke_pri() {
        let entry_with_multiple_ke_pri: &str = "\
        <ent_seq>1591050</ent_seq>\
        <k_ele><keb>大きい</keb><ke_pri>ichi1</ke_pri><ke_pri>news1</ke_pri><ke_pri>gai1</ke_pri></k_ele>\
        <r_ele><reb>おおきい</reb></r_ele>\
        <sense><pos>&adj-i;</pos><gloss>big</gloss><gloss>large</gloss></sense>\
        ";
        let actual = Parser::extract_all_tags(
            entry_with_multiple_ke_pri,
            Parser::KE_PRI_START,
            Parser::KE_PRI_END,
        );
        assert_eq!(actual, Some(vec!["ichi1", "news1", "gai1"]));
    }

    #[test]
    fn should_extract_all_gloss() {
        let entry_with_multiple_gloss: &str = "\
        <ent_seq>1591090</ent_seq>\
        <k_ele><keb>引く</keb><ke_pri>ichi1</ke_pri><ke_pri>news1</ke_pri></k_ele>\
        <r_ele><reb>ひく</reb></r_ele>\
        <sense><pos>&v5k;</pos><pos>&vt;</pos><gloss>to pull</gloss><gloss>to draw</gloss><gloss>to catch (a cold)</gloss><gloss>to subtract</gloss></sense>\
        ";
        let actual = Parser::extract_all_tags(
            entry_with_multiple_gloss,
            Parser::GLOSS_START,
            Parser::GLOSS_END,
        );

        assert_eq!(
            actual,
            Some(vec![
                "to pull",
                "to draw",
                "to catch (a cold)",
                "to subtract"
            ])
        )
    }

    #[test]
    fn should_extract_all_pos() {
        let entry_with_multiple_pos: &str = "\
        <ent_seq>1587500</ent_seq>\
        <k_ele><keb>出来る</keb><ke_pri>ichi1</ke_pri><ke_pri>news1</ke_pri></k_ele>\
        <r_ele><reb>できる</reb></r_ele>\
        <sense><pos>&v1;</pos><pos>&vi;</pos><pos>&aux-v;</pos><gloss>to be able to (do)</gloss><gloss>to be ready</gloss></sense>\
        ";

        let actual =
            Parser::extract_all_tags(entry_with_multiple_pos, Parser::POS_START, Parser::POS_END);
        assert_eq!(actual, Some(vec!["&v1;", "&vi;", "&aux-v;"]));
    }

    #[test]
    fn should_extract_none_when_no_matching_tags() {
        let malformed_string: &str = "\
        <ent_ee>1587500</ent_ee>\
        <k_el1><keb>出来る</keb><ke_pro>ichi1</ke_proo><ke_prpp>news1</ke_prooo></k_el1>\
        <r_ele1><reb>できる</reb2></r_ele>\
        <sense1231><pos123>&v1;</pos12><pos312>&vi;</pos><pos>&aux-v;</pos123><gloss123>to be able to (do)</gloss><gloss>to be ready</gloss></sense>\
        ";

        let actual = Parser::extract_all_tags(malformed_string, Parser::POS_START, Parser::POS_END);

        assert_eq!(actual, None)
    }

    fn make_jmdict_file(contents: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        writeln!(file, "{contents}").expect("failed to write to temp file");
        file
    }

    #[test]
    fn should_parse_single_entry_from_file() {
        let file = make_jmdict_file(
            "<entry>
<ent_seq>1358280</ent_seq>
<k_ele><keb>食べる</keb><ke_pri>ichi1</ke_pri></k_ele>
<r_ele><reb>たべる</reb></r_ele>
<sense><pos>&v1;</pos><gloss>to eat</gloss></sense>
</entry>",
        );

        let actual = Parser::parse_xml(
            file.path()
                .to_str()
                .expect("temp file path should be valid UTF-8"),
        )
        .expect("parse_xml should succeed for a well-formed file");

        assert_eq!(actual.words.len(), 1);

        let word = &actual.words[0];
        assert_eq!(word.id, "1358280");
        assert_eq!(word.kanji, Some(vec!["食べる".to_string()]));
        assert_eq!(word.kana_reading, vec!["たべる".to_string()]);
        assert_eq!(word.use_frequency, Some(vec![UseFrequency::Ichi(1)]));
        assert_eq!(word.english_meaning, vec!["to eat".to_string()]);
        assert_eq!(
            word.part_of_speech,
            vec![PartOfSpeech::Verb(VerbType::Ichidan)]
        );
    }

    #[test]
    fn should_parse_multiple_entries_from_file() {
        let file = make_jmdict_file(
            "<entry>
<ent_seq>1358280</ent_seq>
<k_ele><keb>食べる</keb></k_ele>
<r_ele><reb>たべる</reb></r_ele>
<sense><pos>&v1;</pos><gloss>to eat</gloss></sense>
</entry>
<entry>
<ent_seq>1587040</ent_seq>
<k_ele><keb>今日</keb></k_ele>
<r_ele><reb>きょう</reb></r_ele>
<sense><pos>&n;</pos><gloss>today</gloss></sense>
</entry>",
        );

        let actual = Parser::parse_xml(
            file.path()
                .to_str()
                .expect("temp file path should be valid UTF-8"),
        )
        .expect("parse_xml should succeed for a well-formed file");

        let ids: Vec<&str> = actual.words.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids, vec!["1358280", "1587040"]);
    }

    #[test]
    fn should_skip_entry_missing_ent_seq() {
        let file = make_jmdict_file(
            "<entry>
<k_ele><keb>食べる</keb></k_ele>
<r_ele><reb>たべる</reb></r_ele>
<sense><pos>&v1;</pos><gloss>to eat</gloss></sense>
</entry>",
        );

        let actual = Parser::parse_xml(
            file.path()
                .to_str()
                .expect("temp file path should be valid UTF-8"),
        )
        .expect("parse_xml should succeed even for a skippable entry");

        assert!(actual.words.is_empty());
    }

    #[test]
    fn should_return_io_error_when_file_does_not_exist() {
        let actual = Parser::parse_xml("this_file_does_not_exist.xml");

        assert!(matches!(actual, Err(ParserError::Io(_))));
    }
}
