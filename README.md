# Miso Jisho

## Overview
This project currently parses dictionary entries from a single XML file. This is a WIP but can be run in your terminal after downloading. This currently only supports Japanese to English definitions. You can search either in English or in Japanese.


### How To Use 
1. Clone repo 
2. Navigate inside the project either from your terminal or IDE terminal. 
3. `cargo run` 
4. Search 
5. Top 3 results will appear (If there is more than 1)
6. If there are more results you can press enter to see the rest.

### Workflow / How this works
**Parsing the XML file:** 
To parse the xml file we need to grab the contents of each `<entry>`. Each `<entry>` has tags which represent kanji, kana, etc. (`<entry>somecontent</entry>`). Inside the entry I parse `entry_sequence_id`, `kanji`, `use_frequency`, `english_meaning`, and `part_of_speech`. From the dictionary form of a verb, I generate conjugations together to build our `JpToEnglishWord`.

`use_frequency` and `part_of_speech` are mapped to their own respective types to easily build off of them rather than have just a string. `UseFrequency` helps me sort relevant search results. `PartOfSpeech` lets me identify its part of speech, and if it is a verb, tell what kind of conjugations are needed.

**Japanese to English Dictionary**
All of the parsed entries are added in as `JpToEnglishWord`. These can be searched either in English or in Japanese. `UseFrequency` is used to help sort through the results along with whether the search term is an exact match. Exact matches and higher use frequency are both weighted highest. Words may have multiple definitions so going off of the first definition helps sorting as well. 

| XML Tag      | Meaning              |
| ------------ | -------------------- |
| `<entry>`    | Word content start   |
| `<ent_seq>`  | Entry ID             |
| `<keb>`      | Kanji reading        |
| `<ke_pri>`   | `UseFrequency`        |
| `<reb>`      | Kana reading         |
| `<pos>`      | `PartOfSpeech`        |
| `<gloss>`    | English meaning      |

Example entry below:

```
<entry>
<ent_seq>1591430</ent_seq>
<k_ele>
<keb>走る</keb>
<ke_pri>ichi1</ke_pri>
<ke_pri>news1</ke_pri>
</k_ele>
<r_ele>
<reb>はしる</reb>
</r_ele>
<sense>
<pos>&v5r;</pos>
<pos>&vi;</pos>
<gloss>to run</gloss>
</sense>
</entry>

```

### Plans 
I plan to make this a Tauri project.
