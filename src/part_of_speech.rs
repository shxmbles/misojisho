#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PartOfSpeech {
    Verb(VerbType),
    Noun,
    Adjective,
    Adverbs,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum VerbType {
    Ichidan,
    Godan,
    Irregular,
    Transitive,
    Intransitive,
}

const ICHIDAN: &[&str] = &["&v1;", "&v1-s;", "&vz;"];
/// Matches any Godan verb code (`&v5aru;`, `&v5b;`, `&v5g;`, `&v5k;`, `&v5k-s;`,
/// `&v5m;`, `&v5n;`, `&v5r;`, `&v5r-i;`, `&v5s;`, `&v5t;`, `&v5u;`, `&v5u-s;`,
/// `&v5uru;`) via prefix - every Godan sub-code in JMdict starts with "&v5"
const GODAN: &str = "&v5";
const IRREGULAR: &[&str] = &["&vk;", "&vn;", "&vr;", "&vs;", "&vs-c;", "&vs-i;", "&vs-s;"];

/// Matches any noun code (`&n;`, `&n-adv;`, `&n-pr;`, `&n-pref;`, `&n-suf;`,`&n-t;`) via prefix.
const NOUNS: &str = "&n";

/// Matches any adjective code (`&adj-i;`, `&adj-ix;`, `&adj-kari;`, `&adj-ku;`,
/// `&adj-na;`, `&adj-nari;`, `&adj-no;`, `&adj-pn;`, `&adj-shiku;`, `&adj-t;`,
/// `&adj-f;`) via prefix. `&aux-adj;` is handled separately since it doesn't
/// share this prefix.
const ADJECTIVES: &str = "&adj";
const AUX_ADJECTIVE: &str = "&aux-adj";

const TRANSITIVE: &str = "&vt;";
const INTRANSITIVE: &str = "&vi;";

/// Matches any adverb code (`&adv;`, `&adv-to;`) via prefix.
const ADVERBS: &str = "&adv";

impl PartOfSpeech {
    /// Takes a <pos> tag value and converts to a `PartOfSpeech` Object
    /// # Examples
    /// ```
    /// use misojisho::part_of_speech::PartOfSpeech;
    ///
    /// let actual = PartOfSpeech::from_pos_code("&n");
    /// assert_eq!(actual, Some(PartOfSpeech::Noun));
    /// ```
    pub fn from_pos_code(value: &str) -> Option<Self> {
        if ICHIDAN.contains(&value) {
            Some(PartOfSpeech::Verb(VerbType::Ichidan))
        } else if value.starts_with(GODAN) {
            Some(PartOfSpeech::Verb(VerbType::Godan))
        } else if IRREGULAR.contains(&value) {
            Some(PartOfSpeech::Verb(VerbType::Irregular))
        } else if value == TRANSITIVE {
            Some(PartOfSpeech::Verb(VerbType::Transitive))
        } else if value == INTRANSITIVE {
            Some(PartOfSpeech::Verb(VerbType::Intransitive))
        } else if value.starts_with(NOUNS) {
            Some(PartOfSpeech::Noun)
        } else if value.starts_with(ADJECTIVES) || value.starts_with(AUX_ADJECTIVE) {
            Some(PartOfSpeech::Adjective)
        } else if value == ADVERBS {
            Some(PartOfSpeech::Adverbs)
        } else {
            None
        }
    }
}

impl std::fmt::Display for VerbType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Ichidan => "Ichidan Verb",
            Self::Godan => "Godan Verb",
            Self::Irregular => "Irregular Verb",
            Self::Transitive => "Transitive Verb",
            Self::Intransitive => "Intransitive Verb",
        };

        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // "&v1;", "&v1-s;", "&vz;"
    #[test]
    fn should_map_v1_to_ichidan() {
        let actual = PartOfSpeech::from_pos_code("&v1;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Ichidan)))
    }
    #[test]
    fn should_map_v1_s_to_ichidan() {
        let actual = PartOfSpeech::from_pos_code("&v1-s;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Ichidan)))
    }

    #[test]
    fn should_map_vz_to_ichidan() {
        let actual = PartOfSpeech::from_pos_code("&vz;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Ichidan)))
    }

    // "&v5aru;", "&v5b;", "&v5g;", "&v5k;", "&v5k-s;", "&v5m;", "&v5n;", "&v5r;", "&v5r-i;", "&v5s;",
    // "&v5t;", "&v5u;", "&v5u-s;", "&v5uru;",
    #[test]
    fn should_map_str_starting_with_v5_to_godan() {
        let actual = PartOfSpeech::from_pos_code("&v5");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Godan)))
    }

    // "&vk;", "&vn;", "&vr;", "&vs;", "&vs-c;", "&vs-i;", "&vs-s;"
    #[test]
    fn should_map_vk_to_irregular() {
        let actual = PartOfSpeech::from_pos_code("&vk;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Irregular)))
    }

    #[test]
    fn should_map_vn_to_irregular() {
        let actual = PartOfSpeech::from_pos_code("&vn;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Irregular)))
    }

    #[test]
    fn should_map_vr_to_irregular() {
        let actual = PartOfSpeech::from_pos_code("&vr;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Irregular)))
    }

    #[test]
    fn should_map_vs_to_irregular() {
        let actual = PartOfSpeech::from_pos_code("&vs;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Irregular)))
    }

    //"&vs-c;"
    #[test]
    fn should_map_vs_c_to_irregular() {
        let actual = PartOfSpeech::from_pos_code("&vs-c;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Irregular)))
    }

    #[test]
    fn should_map_vs_i_to_irregular() {
        let actual = PartOfSpeech::from_pos_code("&vs-i;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Irregular)))
    }

    #[test]
    fn should_map_vs_s_to_irregular() {
        let actual = PartOfSpeech::from_pos_code("&vs-s;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Irregular)))
    }

    // "&n;", "&n-adv;", "&n-pr;", "&n-pref;", "&n-suf;", "&n-t;"
    #[test]
    fn should_map_str_starting_with_n_to_noun() {
        let actual = PartOfSpeech::from_pos_code("&n");
        assert_eq!(actual, Some(PartOfSpeech::Noun))
    }

    #[test]
    fn should_return_none_when_no_valid_str() {
        let actual = PartOfSpeech::from_pos_code("chaewon");
        assert_eq!(actual, None);
    }

    #[test]
    fn should_map_vt_to_transitive() {
        let actual = PartOfSpeech::from_pos_code("&vt;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Transitive)));
    }

    #[test]
    fn should_map_vi_to_intransitive() {
        let actual = PartOfSpeech::from_pos_code("&vi;");
        assert_eq!(actual, Some(PartOfSpeech::Verb(VerbType::Intransitive)));
    }

    #[test]
    fn should_map_to_adj_from_adj_prefix() {
        let actual = PartOfSpeech::from_pos_code("&adj");
        assert_eq!(actual, Some(PartOfSpeech::Adjective))
    }

    #[test]
    fn should_map_to_adj_from_aux_adj() {
        let actual = PartOfSpeech::from_pos_code("&aux-adj");
        assert_eq!(actual, Some(PartOfSpeech::Adjective))
    }

    #[test]
    fn should_map_to_adverb_from_adv_prefix() {
        let actual = PartOfSpeech::from_pos_code("&adv");
        assert_eq!(actual, Some(PartOfSpeech::Adverbs))
    }
}
