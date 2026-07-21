use crate::part_of_speech::VerbType;

#[derive(Debug, PartialEq)]
pub struct VerbConjugations {
    pub formal: Formal,
    pub informal: Informal,
    pub other: Other,
}

#[derive(Debug, PartialEq)]
pub struct Formal {
    pub present: String,
    pub negative: String,
    pub past: String,
    pub negative_past: String,
}

#[derive(Debug, PartialEq)]
pub struct Informal {
    pub present: String,
    pub negative: String,
    pub past: String,
    pub negative_past: String,
}

#[derive(Debug, PartialEq)]
pub struct Other {
    pub te: String,
    pub imperative: String,
    pub volitional: String,
    pub passive: String,
    pub causative: String,
    pub hypothetical: String,
    pub potential: String,
    pub polite_volitional: String,
}

impl VerbConjugations {
    /// Conjugates verbs based upon their dictionary form.
    ///
    /// Returns `None` in three cases:
    /// 1. `Transitive`/`Intransitive` verbs are also tagged with their actual
    ///    conjugation class (e.g. Ichidan, Godan) elsewhere, so there's no need
    ///    to conjugate them twice under this variant.
    /// 2. JMdict marks き-ending words as verbs, which is technically correct,
    ///    but they're already in a conjugated form..conjugating them again
    ///    would be unnecessary, so we skip them.
    /// 3. There is a malformed entry.
    pub fn conjugate(verb_type: VerbType, dict_form: &str) -> Option<VerbConjugations> {
        match verb_type {
            VerbType::Ichidan => Self::conjugate_ichidan(dict_form),
            VerbType::Godan => Self::conjugate_godan(dict_form),
            VerbType::Irregular => Self::conjugate_irregular(dict_form),
            VerbType::Transitive => None,
            VerbType::Intransitive => None,
        }
    }

    pub fn conjugate_ichidan(dict_form: &str) -> Option<VerbConjugations> {
        let stem = dict_form.strip_suffix('る')?;
        Some(VerbConjugations {
            formal: Self::formal_ichidan(stem),
            informal: Self::informal_ichidan(stem),
            other: Self::other_ichidan(stem),
        })
    }

    pub fn conjugate_godan(dict_form: &str) -> Option<VerbConjugations> {
        let mut chars = dict_form.chars();
        let verb_ending = chars.next_back()?;
        let stem = chars.as_str();

        match verb_ending {
            'す' => Some(Self::su_conjugation(stem)),
            'く' => Some(Self::ku_conjugation(stem)),
            'ぐ' => Some(Self::gu_conjugation(stem)),
            'ぶ' => Some(Self::bu_conjugation(stem)),
            'ぬ' => Some(Self::nu_conjugation(stem)),
            'む' => Some(Self::mu_conjugation(stem)),
            'る' => Some(Self::ru_conjugation(stem)),
            'う' => Some(Self::u_conjugation(stem)),
            'つ' => Some(Self::tsu_conjugation(stem)),
            'き' => None,
            _ => None,
        }
    }

    /// Conjugates an irregular Japanese verb (する or くる).
    ///
    /// Handles bare forms (`する`, `来る`, `くる`) as well as compound verbs
    /// ending in `する` (e.g. 勉強する) or `くる`. The stem is derived by
    /// stripping the final verb suffix using byte-safe slicing, since each
    /// kana character occupies exactly 3 bytes in UTF-8.
    pub fn conjugate_irregular(dict_form: &str) -> Option<VerbConjugations> {
        let normalized = dict_form.replace("来る", "くる").replace("為る", "する");

        match normalized.as_str() {
            "する" => Some(Self::suru_irregular("")),
            "くる" => Some(Self::kuru_irregular("")),
            s if s.ends_with("する") => {
                let stem = &s[..s.len() - "する".len()];
                Some(Self::suru_irregular(stem))
            }
            s if s.ends_with("くる") => {
                let stem = &s[..s.len() - "くる".len()];
                Some(Self::kuru_irregular(stem))
            }
            _ => None,
        }
    }

    /// Generates the formal conjugations for an ichidan verb.
    ///
    /// # Arguments
    /// * `dict_form` - The dictionary form of a word to conjugate
    ///
    /// # Returns
    /// A [`Formal`] struct containing present, negative, past, and negative past forms.
    fn formal_ichidan(stem: &str) -> Formal {
        Formal {
            present: format!("{}ます", stem),
            negative: format!("{}ません", stem),
            past: format!("{}ました", stem),
            negative_past: format!("{}ませんでした", stem),
        }
    }

    /// Generates the informal conjugations for an ichidan verb.
    ///
    /// # Arguments
    /// * `dict_form` - The dictionary form of a word to conjugate
    ///
    /// # Returns
    /// An [`Informal`] struct containing present, negative, past, and negative past forms.
    fn informal_ichidan(stem: &str) -> Informal {
        Informal {
            present: format!("{}る", stem),
            negative: format!("{}ない", stem),
            past: format!("{}た", stem),
            negative_past: format!("{}なかった", stem),
        }
    }

    /// Generates the "other" conjugations for an ichidan verb.
    ///
    /// # Arguments
    /// * `dict_form` - The dictionary form of a word to conjugate
    ///
    /// # Returns
    /// An [`Other`] struct containing te, imperative, volitional, passive,
    /// causative, hypothetical, potential, and polite volitional forms.
    fn other_ichidan(stem: &str) -> Other {
        Other {
            te: format!("{}て", stem),
            imperative: format!("{}ろ", stem),
            volitional: format!("{}よう", stem),
            passive: format!("{}られる", stem),
            causative: format!("{}させる", stem),
            hypothetical: format!("{}れば", stem),
            potential: format!("{}られる", stem),
            polite_volitional: format!("{}ましょう", stem),
        }
    }

    // MARK:  godan

    fn su_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::su_formal(stem),
            informal: Self::su_informal(stem),
            other: Self::su_other(stem),
        }
    }

    fn ku_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::ku_formal(&stem),
            informal: Self::ku_informal(&stem),
            other: Self::ku_other(&stem),
        }
    }

    fn gu_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::gu_formal(&stem),
            informal: Self::gu_informal(&stem),
            other: Self::gu_other(&stem),
        }
    }

    fn bu_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::bu_formal(stem),
            informal: Self::bu_informal(stem),
            other: Self::bu_other(stem),
        }
    }

    /// Conjugates a ぬ-ending godan verb (e.g. 死ぬ).
    fn nu_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::nu_formal(stem),
            informal: Self::nu_informal(stem),
            other: Self::nu_other(stem),
        }
    }

    /// Conjugates a む-ending godan verb (e.g. 頼む).
    fn mu_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::mu_formal(stem),
            informal: Self::mu_informal(stem),
            other: Self::mu_other(stem),
        }
    }

    /// Conjugates a る-ending godan verb (e.g. 作る). Note: not to be confused with ichidan る verbs.
    fn ru_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::ru_formal(stem),
            informal: Self::ru_informal(stem),
            other: Self::ru_other(stem),
        }
    }

    /// Conjugates a う-ending godan verb (e.g. 買う).
    fn u_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::u_formal(stem),
            informal: Self::u_informal(stem),
            other: Self::u_other(stem),
        }
    }

    /// Conjugates a つ-ending godan verb (e.g. 撃つ).
    fn tsu_conjugation(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::tsu_formal(stem),
            informal: Self::tsu_informal(stem),
            other: Self::tsu_other(stem),
        }
    }

    fn su_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}します", stem),
            negative: format!("{}しません", stem),
            past: format!("{}しました", stem),
            negative_past: format!("{}しませんでした", stem),
        }
    }

    fn su_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}す", stem),
            negative: format!("{}さない", stem),
            past: format!("{}した", stem),
            negative_past: format!("{}さなかった", stem),
        }
    }

    fn su_other(stem: &str) -> Other {
        Other {
            te: format!("{}して", stem),
            imperative: format!("{}せ", stem),
            volitional: format!("{}そう", stem),
            passive: format!("{}される", stem),
            causative: format!("{}させる", stem),
            hypothetical: format!("{}せば", stem),
            potential: format!("{}せる", stem),
            polite_volitional: format!("{}しましょう", stem),
        }
    }

    fn ku_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}きます", stem),
            negative: format!("{}きません", stem),
            past: format!("{}きました", stem),
            negative_past: format!("{}きませんでした", stem),
        }
    }

    fn ku_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}く", stem),
            negative: format!("{}かない", stem),
            past: format!("{}いた", stem),
            negative_past: format!("{}かなかった", stem),
        }
    }

    fn ku_other(stem: &str) -> Other {
        Other {
            te: format!("{}いて", stem),
            imperative: format!("{}け", stem),
            volitional: format!("{}こう", stem),
            passive: format!("{}かれる", stem),
            causative: format!("{}かせる", stem),
            hypothetical: format!("{}けば", stem),
            potential: format!("{}ける", stem),
            polite_volitional: format!("{}きましょう", stem),
        }
    }

    fn gu_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}ぎます", stem),
            negative: format!("{}ぎません", stem),
            past: format!("{}ぎました", stem),
            negative_past: format!("{}ぎませんでした", stem),
        }
    }

    fn gu_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}ぐ", stem),
            negative: format!("{}がない", stem),
            past: format!("{}いだ", stem),
            negative_past: format!("{}がなかった", stem),
        }
    }

    fn gu_other(stem: &str) -> Other {
        Other {
            te: format!("{}いで", stem),
            imperative: format!("{}げ", stem),
            volitional: format!("{}ごう", stem),
            passive: format!("{}がれる", stem),
            causative: format!("{}がせる", stem),
            hypothetical: format!("{}げば", stem),
            potential: format!("{}げる", stem),
            polite_volitional: format!("{}ぎましょう", stem),
        }
    }

    fn bu_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}びます", stem),
            negative: format!("{}びません", stem),
            past: format!("{}びました", stem),
            negative_past: format!("{}びませんでした", stem),
        }
    }

    fn bu_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}ぶ", stem),
            negative: format!("{}ばない", stem),
            past: format!("{}んだ", stem),
            negative_past: format!("{}ばなかった", stem),
        }
    }

    fn bu_other(stem: &str) -> Other {
        Other {
            te: format!("{}んで", stem),
            imperative: format!("{}べ", stem),
            volitional: format!("{}ぼう", stem),
            passive: format!("{}ばれる", stem),
            causative: format!("{}ばせる", stem),
            hypothetical: format!("{}べば", stem),
            potential: format!("{}べる", stem),
            polite_volitional: format!("{}びましょう", stem),
        }
    }

    fn nu_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}にます", stem),
            negative: format!("{}にません", stem),
            past: format!("{}にました", stem),
            negative_past: format!("{}にませんでした", stem),
        }
    }

    fn nu_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}ぬ", stem),
            negative: format!("{}なない", stem),
            past: format!("{}んだ", stem),
            negative_past: format!("{}ななかった", stem),
        }
    }

    fn nu_other(stem: &str) -> Other {
        Other {
            te: format!("{}んで", stem),
            imperative: format!("{}ね", stem),
            volitional: format!("{}のう", stem),
            passive: format!("{}なれる", stem),
            causative: format!("{}なせる", stem),
            hypothetical: format!("{}ねば", stem),
            potential: format!("{}ねる", stem),
            polite_volitional: format!("{}にましょう", stem),
        }
    }

    fn mu_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}みます", stem),
            negative: format!("{}みません", stem),
            past: format!("{}みました", stem),
            negative_past: format!("{}みませんでした", stem),
        }
    }

    fn mu_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}む", stem),
            negative: format!("{}まない", stem),
            past: format!("{}んだ", stem),
            negative_past: format!("{}まなかった", stem),
        }
    }

    fn mu_other(stem: &str) -> Other {
        Other {
            te: format!("{}んで", stem),
            imperative: format!("{}め", stem),
            volitional: format!("{}もう", stem),
            passive: format!("{}まれる", stem),
            causative: format!("{}ませる", stem),
            hypothetical: format!("{}めば", stem),
            potential: format!("{}める", stem),
            polite_volitional: format!("{}みましょう", stem),
        }
    }

    fn ru_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}ります", stem),
            negative: format!("{}りません", stem),
            past: format!("{}りました", stem),
            negative_past: format!("{}りませんでした", stem),
        }
    }

    fn ru_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}る", stem),
            negative: format!("{}らない", stem),
            past: format!("{}った", stem),
            negative_past: format!("{}らなかった", stem),
        }
    }

    fn ru_other(stem: &str) -> Other {
        Other {
            te: format!("{}って", stem),
            imperative: format!("{}れ", stem),
            volitional: format!("{}ろう", stem),
            passive: format!("{}られる", stem),
            causative: format!("{}らせる", stem),
            hypothetical: format!("{}れば", stem),
            potential: format!("{}れる", stem),
            polite_volitional: format!("{}りましょう", stem),
        }
    }

    fn u_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}います", stem),
            negative: format!("{}いません", stem),
            past: format!("{}いました", stem),
            negative_past: format!("{}いませんでした", stem),
        }
    }

    fn u_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}う", stem),
            negative: format!("{}わない", stem),
            past: format!("{}った", stem),
            negative_past: format!("{}わなかった", stem),
        }
    }

    fn u_other(stem: &str) -> Other {
        Other {
            te: format!("{}って", stem),
            imperative: format!("{}え", stem),
            volitional: format!("{}おう", stem),
            passive: format!("{}われる", stem),
            causative: format!("{}わせる", stem),
            hypothetical: format!("{}えば", stem),
            potential: format!("{}える", stem),
            polite_volitional: format!("{}いましょう", stem),
        }
    }

    fn tsu_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}ちます", stem),
            negative: format!("{}ちません", stem),
            past: format!("{}ちました", stem),
            negative_past: format!("{}ちませんでした", stem),
        }
    }

    fn tsu_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}つ", stem),
            negative: format!("{}たない", stem),
            past: format!("{}った", stem),
            negative_past: format!("{}たなかった", stem),
        }
    }

    fn tsu_other(stem: &str) -> Other {
        Other {
            te: format!("{}って", stem),
            imperative: format!("{}て", stem),
            volitional: format!("{}とう", stem),
            passive: format!("{}たれる", stem),
            causative: format!("{}たせる", stem),
            hypothetical: format!("{}てば", stem),
            potential: format!("{}てる", stem),
            polite_volitional: format!("{}ちましょう", stem),
        }
    }

    // MARK: Irregular

    fn suru_irregular(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::suru_formal(stem),
            informal: Self::suru_informal(stem),
            other: Self::suru_other(stem),
        }
    }

    fn kuru_irregular(stem: &str) -> VerbConjugations {
        VerbConjugations {
            formal: Self::kuru_formal(stem),
            informal: Self::kuru_informal(stem),
            other: Self::kuru_other(stem),
        }
    }

    fn suru_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}します", stem),
            negative: format!("{}しません", stem),
            past: format!("{}しました", stem),
            negative_past: format!("{}しませんでした", stem),
        }
    }

    fn suru_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}する", stem),
            negative: format!("{}しない", stem),
            past: format!("{}した", stem),
            negative_past: format!("{}しなかった", stem),
        }
    }

    fn suru_other(stem: &str) -> Other {
        Other {
            te: format!("{}して", stem),
            imperative: format!("{}しろ", stem),
            volitional: format!("{}しよう", stem),
            passive: format!("{}される", stem),
            causative: format!("{}させる", stem),
            hypothetical: format!("{}すれば", stem),
            potential: format!("{}できる", stem),
            polite_volitional: format!("{}しましょう", stem),
        }
    }

    fn kuru_formal(stem: &str) -> Formal {
        Formal {
            present: format!("{}きます", stem),
            negative: format!("{}きません", stem),
            past: format!("{}きました", stem),
            negative_past: format!("{}きませんでした", stem),
        }
    }

    fn kuru_informal(stem: &str) -> Informal {
        Informal {
            present: format!("{}くる", stem),
            negative: format!("{}こない", stem),
            past: format!("{}きた", stem),
            negative_past: format!("{}こなかった", stem),
        }
    }

    fn kuru_other(stem: &str) -> Other {
        Other {
            te: format!("{}きて", stem),
            imperative: format!("{}こい", stem),
            volitional: format!("{}こよう", stem),
            passive: format!("{}こられる", stem),
            causative: format!("{}こさせる", stem),
            hypothetical: format!("{}くれば", stem),
            potential: format!("{}こられる", stem),
            polite_volitional: format!("{}きましょう", stem),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_conjugate_ichidan() {
        let actual = VerbConjugations::conjugate(VerbType::Ichidan, "認める");
        let expected = Some(VerbConjugations {
            formal: Formal {
                present: String::from("認めます"),
                negative: String::from("認めません"),
                past: String::from("認めました"),
                negative_past: String::from("認めませんでした"),
            },
            informal: Informal {
                present: String::from("認める"),
                negative: String::from("認めない"),
                past: String::from("認めた"),
                negative_past: String::from("認めなかった"),
            },
            other: Other {
                te: String::from("認めて"),
                imperative: String::from("認めろ"),
                volitional: String::from("認めよう"),
                passive: String::from("認められる"),
                causative: String::from("認めさせる"),
                hypothetical: String::from("認めれば"),
                potential: String::from("認められる"),
                polite_volitional: String::from("認めましょう"),
            },
        });

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_godan() {
        let actual = VerbConjugations::conjugate(VerbType::Godan, "抱く");
        let expected = Some(VerbConjugations {
            formal: Formal {
                present: String::from("抱きます"),
                negative: String::from("抱きません"),
                past: String::from("抱きました"),
                negative_past: String::from("抱きませんでした"),
            },
            informal: Informal {
                present: String::from("抱く"),
                negative: String::from("抱かない"),
                past: String::from("抱いた"),
                negative_past: String::from("抱かなかった"),
            },
            other: Other {
                te: String::from("抱いて"),
                imperative: String::from("抱け"),
                volitional: String::from("抱こう"),
                passive: String::from("抱かれる"),
                causative: String::from("抱かせる"),
                hypothetical: String::from("抱けば"),
                potential: String::from("抱ける"),
                polite_volitional: String::from("抱きましょう"),
            },
        });

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_irregular() {
        let actual = VerbConjugations::conjugate(VerbType::Irregular, "旅行する");
        let expected = Some(VerbConjugations {
            formal: Formal {
                present: String::from("旅行します"),
                negative: String::from("旅行しません"),
                past: String::from("旅行しました"),
                negative_past: String::from("旅行しませんでした"),
            },
            informal: Informal {
                present: String::from("旅行する"),
                negative: String::from("旅行しない"),
                past: String::from("旅行した"),
                negative_past: String::from("旅行しなかった"),
            },
            other: Other {
                te: String::from("旅行して"),
                imperative: String::from("旅行しろ"),
                volitional: String::from("旅行しよう"),
                passive: String::from("旅行される"),
                causative: String::from("旅行させる"),
                hypothetical: String::from("旅行すれば"),
                potential: String::from("旅行できる"),
                polite_volitional: String::from("旅行しましょう"),
            },
        });

        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_none_when_transitive() {
        let actual = VerbConjugations::conjugate(VerbType::Transitive, "教える");
        assert_eq!(actual, None)
    }

    #[test]
    fn should_conjugate_none_when_intransitive() {
        let actual = VerbConjugations::conjugate(VerbType::Intransitive, "歩く");
        assert_eq!(actual, None)
    }

    #[test]
    fn should_conjugate_ru_ichidan() {
        let expected = VerbConjugations {
            formal: Formal {
                present: String::from("食べます"),
                negative: String::from("食べません"),
                past: String::from("食べました"),
                negative_past: String::from("食べませんでした"),
            },
            informal: Informal {
                present: String::from("食べる"),
                negative: String::from("食べない"),
                past: String::from("食べた"),
                negative_past: String::from("食べなかった"),
            },
            other: Other {
                te: String::from("食べて"),
                imperative: String::from("食べろ"),
                volitional: String::from("食べよう"),
                passive: String::from("食べられる"),
                causative: String::from("食べさせる"),
                hypothetical: String::from("食べれば"),
                potential: String::from("食べられる"),
                polite_volitional: String::from("食べましょう"),
            },
        };

        let actual = VerbConjugations::conjugate_ichidan("食べる");

        assert_eq!(actual, Some(expected))
    }

    #[test]
    fn should_not_conjugate_malformed_ichidan() {
        let actual = VerbConjugations::conjugate_ichidan("食べわ");

        assert_eq!(actual, None)
    }

    #[test]
    fn should_conjugate_godan_dispatches() {
        let cases: &[(&str, fn(&str) -> VerbConjugations)] = &[
            ("話す", VerbConjugations::su_conjugation),
            ("働く", VerbConjugations::ku_conjugation),
            ("稼ぐ", VerbConjugations::gu_conjugation),
            ("運ぶ", VerbConjugations::bu_conjugation),
            ("死ぬ", VerbConjugations::nu_conjugation),
            ("頼む", VerbConjugations::mu_conjugation),
            ("作る", VerbConjugations::ru_conjugation),
            ("買う", VerbConjugations::u_conjugation),
            ("撃つ", VerbConjugations::tsu_conjugation),
        ];

        for (dict_form, expected_fn) in cases {
            let mut stem = dict_form.to_string();
            stem.pop();

            let actual = VerbConjugations::conjugate_godan(dict_form);
            let expected = expected_fn(&stem);

            assert_eq!(actual, Some(expected), "wrong dispatch for {}", dict_form);
        }
    }

    #[test]
    fn should_not_conjugate_malformed_godan() {
        let actual = VerbConjugations::conjugate_godan("歩き");

        assert_eq!(actual, None)
    }

    #[test]
    fn should_conjugate_su() {
        let mut stem = String::from("話す");
        stem.pop();
        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}します", stem),
                negative: format!("{}しません", stem),
                past: format!("{}しました", stem),
                negative_past: format!("{}しませんでした", stem),
            },
            informal: Informal {
                present: format!("{}す", stem),
                negative: format!("{}さない", stem),
                past: format!("{}した", stem),
                negative_past: format!("{}さなかった", stem),
            },
            other: Other {
                te: format!("{}して", stem),
                imperative: format!("{}せ", stem),
                volitional: format!("{}そう", stem),
                passive: format!("{}される", stem),
                causative: format!("{}させる", stem),
                hypothetical: format!("{}せば", stem),
                potential: format!("{}せる", stem),
                polite_volitional: format!("{}しましょう", stem),
            },
        };

        let actual = VerbConjugations::su_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn shoudl_conjugate_ku() {
        let mut stem = String::from("働く");
        stem.pop();
        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}きます", stem),
                negative: format!("{}きません", stem),
                past: format!("{}きました", stem),
                negative_past: format!("{}きませんでした", stem),
            },
            informal: Informal {
                present: format!("{}く", stem),
                negative: format!("{}かない", stem),
                past: format!("{}いた", stem),
                negative_past: format!("{}かなかった", stem),
            },
            other: Other {
                te: format!("{}いて", stem),
                imperative: format!("{}け", stem),
                volitional: format!("{}こう", stem),
                passive: format!("{}かれる", stem),
                causative: format!("{}かせる", stem),
                hypothetical: format!("{}けば", stem),
                potential: format!("{}ける", stem),
                polite_volitional: format!("{}きましょう", stem),
            },
        };
        let actual = VerbConjugations::ku_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_gu() {
        let mut stem = String::from("稼ぐ");
        stem.pop();
        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}ぎます", stem),
                negative: format!("{}ぎません", stem),
                past: format!("{}ぎました", stem),
                negative_past: format!("{}ぎませんでした", stem),
            },
            informal: Informal {
                present: format!("{}ぐ", stem),
                negative: format!("{}がない", stem),
                past: format!("{}いだ", stem),
                negative_past: format!("{}がなかった", stem),
            },
            other: Other {
                te: format!("{}いで", stem),
                imperative: format!("{}げ", stem),
                volitional: format!("{}ごう", stem),
                passive: format!("{}がれる", stem),
                causative: format!("{}がせる", stem),
                hypothetical: format!("{}げば", stem),
                potential: format!("{}げる", stem),
                polite_volitional: format!("{}ぎましょう", stem),
            },
        };
        let actual = VerbConjugations::gu_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_bu() {
        let mut stem = String::from("運ぶ");
        stem.pop();

        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}びます", stem),
                negative: format!("{}びません", stem),
                past: format!("{}びました", stem),
                negative_past: format!("{}びませんでした", stem),
            },
            informal: Informal {
                present: format!("{}ぶ", stem),
                negative: format!("{}ばない", stem),
                past: format!("{}んだ", stem),
                negative_past: format!("{}ばなかった", stem),
            },
            other: Other {
                te: format!("{}んで", stem),
                imperative: format!("{}べ", stem),
                volitional: format!("{}ぼう", stem),
                passive: format!("{}ばれる", stem),
                causative: format!("{}ばせる", stem),
                hypothetical: format!("{}べば", stem),
                potential: format!("{}べる", stem),
                polite_volitional: format!("{}びましょう", stem),
            },
        };

        let actual = VerbConjugations::bu_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_nu() {
        let mut stem = String::from("死ぬ");
        stem.pop();

        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}にます", stem),
                negative: format!("{}にません", stem),
                past: format!("{}にました", stem),
                negative_past: format!("{}にませんでした", stem),
            },
            informal: Informal {
                present: format!("{}ぬ", stem),
                negative: format!("{}なない", stem),
                past: format!("{}んだ", stem),
                negative_past: format!("{}ななかった", stem),
            },
            other: Other {
                te: format!("{}んで", stem),
                imperative: format!("{}ね", stem),
                volitional: format!("{}のう", stem),
                passive: format!("{}なれる", stem),
                causative: format!("{}なせる", stem),
                hypothetical: format!("{}ねば", stem),
                potential: format!("{}ねる", stem),
                polite_volitional: format!("{}にましょう", stem),
            },
        };

        let actual = VerbConjugations::nu_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_mu() {
        let mut stem = String::from("頼む");
        stem.pop();

        let actual: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}みます", stem),
                negative: format!("{}みません", stem),
                past: format!("{}みました", stem),
                negative_past: format!("{}みませんでした", stem),
            },
            informal: Informal {
                present: format!("{}む", stem),
                negative: format!("{}まない", stem),
                past: format!("{}んだ", stem),
                negative_past: format!("{}まなかった", stem),
            },
            other: Other {
                te: format!("{}んで", stem),
                imperative: format!("{}め", stem),
                volitional: format!("{}もう", stem),
                passive: format!("{}まれる", stem),
                causative: format!("{}ませる", stem),
                hypothetical: format!("{}めば", stem),
                potential: format!("{}める", stem),
                polite_volitional: format!("{}みましょう", stem),
            },
        };

        let expected = VerbConjugations::mu_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_ru() {
        let mut stem = String::from("作る");
        stem.pop();

        let actual: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}ります", stem),
                negative: format!("{}りません", stem),
                past: format!("{}りました", stem),
                negative_past: format!("{}りませんでした", stem),
            },
            informal: Informal {
                present: format!("{}る", stem),
                negative: format!("{}らない", stem),
                past: format!("{}った", stem),
                negative_past: format!("{}らなかった", stem),
            },
            other: Other {
                te: format!("{}って", stem),
                imperative: format!("{}れ", stem),
                volitional: format!("{}ろう", stem),
                passive: format!("{}られる", stem),
                causative: format!("{}らせる", stem),
                hypothetical: format!("{}れば", stem),
                potential: format!("{}れる", stem),
                polite_volitional: format!("{}りましょう", stem),
            },
        };

        let expected = VerbConjugations::ru_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_u() {
        let mut stem = String::from("買う");
        stem.pop();

        let actual: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}います", stem),
                negative: format!("{}いません", stem),
                past: format!("{}いました", stem),
                negative_past: format!("{}いませんでした", stem),
            },
            informal: Informal {
                present: format!("{}う", stem),
                negative: format!("{}わない", stem),
                past: format!("{}った", stem),
                negative_past: format!("{}わなかった", stem),
            },
            other: Other {
                te: format!("{}って", stem),
                imperative: format!("{}え", stem),
                volitional: format!("{}おう", stem),
                passive: format!("{}われる", stem),
                causative: format!("{}わせる", stem),
                hypothetical: format!("{}えば", stem),
                potential: format!("{}える", stem),
                polite_volitional: format!("{}いましょう", stem),
            },
        };

        let expected = VerbConjugations::u_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_tsu() {
        let mut stem = String::from("撃つ");
        stem.pop();

        let actual: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: format!("{}ちます", stem),
                negative: format!("{}ちません", stem),
                past: format!("{}ちました", stem),
                negative_past: format!("{}ちませんでした", stem),
            },
            informal: Informal {
                present: format!("{}つ", stem),
                negative: format!("{}たない", stem),
                past: format!("{}った", stem),
                negative_past: format!("{}たなかった", stem),
            },
            other: Other {
                te: format!("{}って", stem),
                imperative: format!("{}て", stem),
                volitional: format!("{}とう", stem),
                passive: format!("{}たれる", stem),
                causative: format!("{}たせる", stem),
                hypothetical: format!("{}てば", stem),
                potential: format!("{}てる", stem),
                polite_volitional: format!("{}ちましょう", stem),
            },
        };

        let expected = VerbConjugations::tsu_conjugation(&stem);
        assert_eq!(actual, expected)
    }

    #[test]
    fn should_conjugate_suru() {
        let actual = VerbConjugations::conjugate_irregular("する");
        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: String::from("します"),
                negative: String::from("しません"),
                past: String::from("しました"),
                negative_past: String::from("しませんでした"),
            },
            informal: Informal {
                present: String::from("する"),
                negative: String::from("しない"),
                past: String::from("した"),
                negative_past: String::from("しなかった"),
            },
            other: Other {
                te: String::from("して"),
                imperative: String::from("しろ"),
                volitional: String::from("しよう"),
                passive: String::from("される"),
                causative: String::from("させる"),
                hypothetical: String::from("すれば"),
                potential: String::from("できる"),
                polite_volitional: String::from("しましょう"),
            },
        };

        assert_eq!(actual, Some(expected))
    }

    #[test]
    fn should_conjugate_kuru() {
        let actual = VerbConjugations::conjugate_irregular("くる");

        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: String::from("きます"),
                negative: String::from("きません"),
                past: String::from("きました"),
                negative_past: String::from("きませんでした"),
            },
            informal: Informal {
                present: String::from("くる"),
                negative: String::from("こない"),
                past: String::from("きた"),
                negative_past: String::from("こなかった"),
            },
            other: Other {
                te: String::from("きて"),
                imperative: String::from("こい"),
                volitional: String::from("こよう"),
                passive: String::from("こられる"),
                causative: String::from("こさせる"),
                hypothetical: String::from("くれば"),
                potential: String::from("こられる"),
                polite_volitional: String::from("きましょう"),
            },
        };

        assert_eq!(actual, Some(expected))
    }

    #[test]
    fn should_conjugate_benkyou_suru() {
        let actual = VerbConjugations::conjugate_irregular("勉強する");
        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: String::from("勉強します"),
                negative: String::from("勉強しません"),
                past: String::from("勉強しました"),
                negative_past: String::from("勉強しませんでした"),
            },
            informal: Informal {
                present: String::from("勉強する"),
                negative: String::from("勉強しない"),
                past: String::from("勉強した"),
                negative_past: String::from("勉強しなかった"),
            },
            other: Other {
                te: String::from("勉強して"),
                imperative: String::from("勉強しろ"),
                volitional: String::from("勉強しよう"),
                passive: String::from("勉強される"),
                causative: String::from("勉強させる"),
                hypothetical: String::from("勉強すれば"),
                potential: String::from("勉強できる"),
                polite_volitional: String::from("勉強しましょう"),
            },
        };

        assert_eq!(actual, Some(expected))
    }

    #[test]
    fn should_conjugate_yatte_kuru() {
        let actual = VerbConjugations::conjugate_irregular("やって来る");

        let expected: VerbConjugations = VerbConjugations {
            formal: Formal {
                present: String::from("やってきます"),
                negative: String::from("やってきません"),
                past: String::from("やってきました"),
                negative_past: String::from("やってきませんでした"),
            },
            informal: Informal {
                present: String::from("やってくる"),
                negative: String::from("やってこない"),
                past: String::from("やってきた"),
                negative_past: String::from("やってこなかった"),
            },
            other: Other {
                te: String::from("やってきて"),
                imperative: String::from("やってこい"),
                volitional: String::from("やってこよう"),
                passive: String::from("やってこられる"),
                causative: String::from("やってこさせる"),
                hypothetical: String::from("やってくれば"),
                potential: String::from("やってこられる"),
                polite_volitional: String::from("やってきましょう"),
            },
        };

        assert_eq!(actual, Some(expected))
    }
}
