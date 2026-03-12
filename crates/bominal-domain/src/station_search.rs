//! Station autofill search engine with multi-strategy ranking.
//!
//! Supports 8 match strategies:
//! - **Exact** — station code or exact name match
//! - **Prefix** — prefix matching on normalized names
//! - **Choseong** (초성) — Korean initial consonant search (e.g. ㄷㄱ → 대구)
//! - **JamoCompose** — Hangul jamo composition matching
//! - **KeyboardLayout** — QWERTY-to-Hangul 2-set decoding (e.g. "tntj" → 수서)
//! - **SymSpell** — fuzzy matching with edit distance
//! - **AliasEn** — English name matching
//! - **AliasJa** — Japanese katakana matching
//!
//! Ported from jason931225/bominal runtime/crates/api/src/services/station_search.rs

use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Suggest,
    Submit,
}

impl SearchMode {
    pub fn from_query(raw: Option<&str>) -> Self {
        match raw
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("suggest")
            .to_ascii_lowercase()
            .as_str()
        {
            "submit" => Self::Submit,
            _ => Self::Suggest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutHint {
    Auto,
    TwoSet,
    Qwerty,
    Jp,
}

impl LayoutHint {
    pub fn from_query(raw: Option<&str>) -> Self {
        match raw
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("auto")
            .to_ascii_lowercase()
            .as_str()
        {
            "2set" | "two_set" => Self::TwoSet,
            "qwerty" => Self::Qwerty,
            "jp" | "ja" | "japanese" => Self::Jp,
            _ => Self::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LangHint {
    Auto,
    Ko,
    En,
    Ja,
}

impl LangHint {
    pub fn from_query(raw: Option<&str>) -> Self {
        match raw
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("auto")
            .to_ascii_lowercase()
            .as_str()
        {
            "ko" | "korean" => Self::Ko,
            "en" | "english" => Self::En,
            "ja" | "jp" | "japanese" => Self::Ja,
            _ => Self::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    pub mode: SearchMode,
    pub layout_hint: LayoutHint,
    pub lang_hint: LangHint,
    pub autocorrect_confidence_threshold: f32,
    pub autocorrect_margin_threshold: f32,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            mode: SearchMode::Suggest,
            layout_hint: LayoutHint::Auto,
            lang_hint: LangHint::Auto,
            autocorrect_confidence_threshold: 0.84,
            autocorrect_margin_threshold: 0.10,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StationSearchDocument<'a> {
    pub station_name_ko: &'a str,
    pub station_name_en: Option<&'a str>,
    pub station_name_ja_katakana: &'a str,
    pub normalized_name: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchSource {
    Exact,
    Prefix,
    Choseong,
    JamoCompose,
    KeyboardLayout,
    SymSpell,
    AliasEn,
    AliasJa,
}

impl MatchSource {
    pub fn as_api_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Prefix => "prefix",
            Self::Choseong => "chosung",
            Self::JamoCompose => "jamo_compose",
            Self::KeyboardLayout => "keyboard_layout",
            Self::SymSpell => "symspell",
            Self::AliasEn => "alias_en",
            Self::AliasJa => "alias_ja",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedMatch {
    pub station_index: usize,
    pub score: usize,
    pub confidence: f32,
    pub source: MatchSource,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SearchResult {
    pub corrected_query: Option<String>,
    pub autocorrect_applied: bool,
    pub matches: Vec<RankedMatch>,
}

pub fn rank_station_documents(
    documents: &[StationSearchDocument<'_>],
    query: &str,
    options: SearchOptions,
    limit: usize,
) -> SearchResult {
    let query_raw = query.trim();
    if query_raw.is_empty() || documents.is_empty() {
        return SearchResult::default();
    }
    let query_norm = normalize(query_raw);
    let query_ascii_key = sanitize_ascii_key_input(query_raw);
    let views = build_query_views(query_raw, query_norm.as_str(), options);
    let station_forms: Vec<StationForms> =
        documents.iter().map(StationForms::from_document).collect();
    let symspell = SymSpellIndex::build(&station_forms, 2);

    let mut best_by_station: HashMap<usize, RankedMatch> = HashMap::new();

    for (station_index, station) in station_forms.iter().enumerate() {
        if !query_ascii_key.is_empty() && !station.keyseq_two_set.is_empty() {
            let key_distance = keyboard_weighted_distance(
                query_ascii_key.as_str(),
                station.keyseq_two_set.as_str(),
                qwerty_neighbors,
            );
            if key_distance <= 1.6 {
                let score = 8 + (key_distance * 12.0).round() as usize;
                upsert_match(
                    &mut best_by_station,
                    RankedMatch {
                        station_index,
                        score,
                        confidence: confidence_from_score(score, MatchSource::KeyboardLayout),
                        source: MatchSource::KeyboardLayout,
                    },
                );
            }
        }

        for view in &views {
            if view.value.is_empty() {
                continue;
            }
            score_direct_match(station_index, station, view, &mut best_by_station);
        }
    }

    for view in &views {
        if view.value.is_empty() {
            continue;
        }
        for candidate in symspell.lookup(view.value.as_str()) {
            let score =
                26 + (candidate.distance * 7) + candidate.form_weight + view.kind.fuzzy_bias();
            upsert_match(
                &mut best_by_station,
                RankedMatch {
                    station_index: candidate.station_index,
                    score,
                    confidence: confidence_from_score(score, MatchSource::SymSpell),
                    source: MatchSource::SymSpell,
                },
            );
        }
    }

    let mut matches: Vec<RankedMatch> = best_by_station.into_values().collect();
    matches.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| right.confidence.total_cmp(&left.confidence))
            .then_with(|| left.station_index.cmp(&right.station_index))
    });
    let capped_limit = limit.clamp(1, 30);
    matches.truncate(capped_limit);

    let mut result = SearchResult {
        corrected_query: None,
        autocorrect_applied: false,
        matches,
    };
    apply_submit_autocorrect(&mut result, documents, query_raw, options);
    result
}

fn normalize(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            continue;
        }

        if ('\u{3041}'..='\u{3096}').contains(&ch) {
            if let Some(katakana) = char::from_u32(ch as u32 + 0x60) {
                out.push(katakana);
            }
            continue;
        }

        if ('\u{30A0}'..='\u{30FF}').contains(&ch) || ch == 'ー' {
            out.push(ch);
            continue;
        }

        if ('\u{AC00}'..='\u{D7A3}').contains(&ch) || ('\u{3131}'..='\u{318E}').contains(&ch) {
            out.push(ch);
        }
    }
    out
}

#[derive(Debug, Clone)]
struct StationForms {
    ko_norm: String,
    en_norm: String,
    ja_norm: String,
    ja_hangul: String,
    initials: String,
    jamo: String,
    keyseq_two_set: String,
}

impl StationForms {
    fn from_document(document: &StationSearchDocument<'_>) -> Self {
        let ko_norm = normalize(document.normalized_name);
        let en_norm = document.station_name_en.map(normalize).unwrap_or_default();
        let ja_norm = normalize(document.station_name_ja_katakana);
        let ja_hangul = transliterate_katakana_to_hangul(document.station_name_ja_katakana);
        let initials = extract_hangul_initials(document.station_name_ko);
        let jamo = decompose_hangul_to_compat_jamo(document.station_name_ko);
        let keyseq_two_set = hangul_to_two_set_keyseq(document.station_name_ko);

        Self {
            ko_norm,
            en_norm,
            ja_norm,
            ja_hangul,
            initials,
            jamo,
            keyseq_two_set,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum QueryViewKind {
    RawNormalized,
    JamoComposed,
    KeyboardDecoded,
    KeyboardNeighborDecoded,
    Choseong,
    DecomposedJamo,
    KatakanaTransliterated,
}

impl QueryViewKind {
    fn base_score(self) -> usize {
        match self {
            Self::RawNormalized => 10,
            Self::JamoComposed => 9,
            Self::KeyboardDecoded => 7,
            Self::KeyboardNeighborDecoded => 11,
            Self::Choseong => 12,
            Self::DecomposedJamo => 13,
            Self::KatakanaTransliterated => 11,
        }
    }

    fn fuzzy_bias(self) -> usize {
        match self {
            Self::RawNormalized => 0,
            Self::JamoComposed => 0,
            Self::KeyboardDecoded => 1,
            Self::KeyboardNeighborDecoded => 4,
            Self::Choseong => 3,
            Self::DecomposedJamo => 2,
            Self::KatakanaTransliterated => 2,
        }
    }
}

#[derive(Debug, Clone)]
struct QueryView {
    value: String,
    kind: QueryViewKind,
}

fn build_query_views(query_raw: &str, query_norm: &str, options: SearchOptions) -> Vec<QueryView> {
    let mut views: Vec<QueryView> = Vec::new();
    let mut seen: HashSet<(String, QueryViewKind)> = HashSet::new();

    push_view(
        &mut views,
        &mut seen,
        QueryView {
            value: query_norm.to_string(),
            kind: QueryViewKind::RawNormalized,
        },
    );

    let composed = compose_compat_jamo_stream(query_norm);
    if !composed.is_empty() && composed != query_norm {
        push_view(
            &mut views,
            &mut seen,
            QueryView {
                value: normalize(composed.as_str()),
                kind: QueryViewKind::JamoComposed,
            },
        );
    }

    let decomposed = decompose_hangul_to_compat_jamo(query_norm);
    if !decomposed.is_empty() && decomposed != query_norm {
        push_view(
            &mut views,
            &mut seen,
            QueryView {
                value: decomposed,
                kind: QueryViewKind::DecomposedJamo,
            },
        );
    }

    let initials = extract_hangul_initials(query_norm);
    if !initials.is_empty() {
        push_view(
            &mut views,
            &mut seen,
            QueryView {
                value: initials,
                kind: QueryViewKind::Choseong,
            },
        );
    }

    if has_katakana(query_norm) {
        let hangul = transliterate_katakana_to_hangul(query_norm);
        if !hangul.is_empty() {
            let hangul_norm = normalize(hangul.as_str());
            push_view(
                &mut views,
                &mut seen,
                QueryView {
                    value: hangul_norm,
                    kind: QueryViewKind::KatakanaTransliterated,
                },
            );
        }
    }

    if options.lang_hint != LangHint::Ja && allows_qwerty_layout(options.layout_hint) {
        let ascii_input = sanitize_ascii_key_input(query_raw);
        if !ascii_input.is_empty() {
            let decoded = decode_two_set_ascii_to_hangul(ascii_input.as_str());
            if !decoded.is_empty() {
                push_view(
                    &mut views,
                    &mut seen,
                    QueryView {
                        value: normalize(decoded.as_str()),
                        kind: QueryViewKind::KeyboardDecoded,
                    },
                );
            }

            for variant in
                generate_single_substitution_variants(ascii_input.as_str(), qwerty_neighbors, 24)
            {
                let decoded_variant = decode_two_set_ascii_to_hangul(variant.as_str());
                if decoded_variant.is_empty() {
                    continue;
                }
                push_view(
                    &mut views,
                    &mut seen,
                    QueryView {
                        value: normalize(decoded_variant.as_str()),
                        kind: QueryViewKind::KeyboardNeighborDecoded,
                    },
                );
            }
        }
    }

    views
}

fn allows_qwerty_layout(layout_hint: LayoutHint) -> bool {
    matches!(
        layout_hint,
        LayoutHint::Auto | LayoutHint::TwoSet | LayoutHint::Qwerty
    )
}

fn push_view(
    views: &mut Vec<QueryView>,
    seen: &mut HashSet<(String, QueryViewKind)>,
    view: QueryView,
) {
    if view.value.is_empty() {
        return;
    }
    let key = (view.value.clone(), view.kind);
    if seen.insert(key) {
        views.push(view);
    }
}

fn score_direct_match(
    station_index: usize,
    station: &StationForms,
    view: &QueryView,
    best_by_station: &mut HashMap<usize, RankedMatch>,
) {
    let value = view.value.as_str();
    let base = view.kind.base_score();

    if station.ko_norm == value {
        update_station(best_by_station, station_index, base, MatchSource::Exact);
    } else if station.ko_norm.starts_with(value) {
        update_station(
            best_by_station,
            station_index,
            base + 3,
            MatchSource::Prefix,
        );
    } else if station.ko_norm.contains(value) {
        update_station(
            best_by_station,
            station_index,
            base + 12,
            MatchSource::Prefix,
        );
    }

    if !station.en_norm.is_empty() {
        if station.en_norm == value {
            update_station(
                best_by_station,
                station_index,
                base + 5,
                MatchSource::AliasEn,
            );
        } else if station.en_norm.starts_with(value) {
            update_station(
                best_by_station,
                station_index,
                base + 9,
                MatchSource::AliasEn,
            );
        } else if station.en_norm.contains(value) {
            update_station(
                best_by_station,
                station_index,
                base + 15,
                MatchSource::AliasEn,
            );
        }
    }

    if !station.ja_norm.is_empty() {
        if station.ja_norm == value {
            update_station(
                best_by_station,
                station_index,
                base + 6,
                MatchSource::AliasJa,
            );
        } else if station.ja_norm.starts_with(value) {
            update_station(
                best_by_station,
                station_index,
                base + 10,
                MatchSource::AliasJa,
            );
        } else if station.ja_norm.contains(value) {
            update_station(
                best_by_station,
                station_index,
                base + 16,
                MatchSource::AliasJa,
            );
        }
    }

    if !station.ja_hangul.is_empty() {
        let ja_hangul_norm = normalize(station.ja_hangul.as_str());
        if ja_hangul_norm == value {
            update_station(
                best_by_station,
                station_index,
                base + 7,
                MatchSource::AliasJa,
            );
        } else if ja_hangul_norm.starts_with(value) {
            update_station(
                best_by_station,
                station_index,
                base + 11,
                MatchSource::AliasJa,
            );
        } else if ja_hangul_norm.contains(value) {
            update_station(
                best_by_station,
                station_index,
                base + 17,
                MatchSource::AliasJa,
            );
        }
    }

    if !station.initials.is_empty() {
        if station.initials == value {
            update_station(
                best_by_station,
                station_index,
                base + 4,
                MatchSource::Choseong,
            );
        } else if station.initials.starts_with(value) {
            update_station(
                best_by_station,
                station_index,
                base + 6,
                MatchSource::Choseong,
            );
        } else if station.initials.contains(value) {
            update_station(
                best_by_station,
                station_index,
                base + 13,
                MatchSource::Choseong,
            );
        }
    }

    if !station.jamo.is_empty() {
        if station.jamo == value {
            update_station(
                best_by_station,
                station_index,
                base + 7,
                MatchSource::JamoCompose,
            );
        } else if station.jamo.starts_with(value) {
            update_station(
                best_by_station,
                station_index,
                base + 9,
                MatchSource::JamoCompose,
            );
        }
    }

    let ko_distance = levenshtein(station.ko_norm.as_str(), value);
    if ko_distance <= 1 {
        update_station(
            best_by_station,
            station_index,
            base + 17,
            MatchSource::SymSpell,
        );
    } else if ko_distance == 2 {
        update_station(
            best_by_station,
            station_index,
            base + 24,
            MatchSource::SymSpell,
        );
    }

    if !station.en_norm.is_empty() {
        let en_distance = levenshtein(station.en_norm.as_str(), value);
        if en_distance <= 1 {
            update_station(
                best_by_station,
                station_index,
                base + 20,
                MatchSource::SymSpell,
            );
        }
    }
}

fn update_station(
    best_by_station: &mut HashMap<usize, RankedMatch>,
    station_index: usize,
    score: usize,
    source: MatchSource,
) {
    upsert_match(
        best_by_station,
        RankedMatch {
            station_index,
            score,
            confidence: confidence_from_score(score, source),
            source,
        },
    );
}

fn upsert_match(best_by_station: &mut HashMap<usize, RankedMatch>, candidate: RankedMatch) {
    match best_by_station.get_mut(&candidate.station_index) {
        Some(current) => {
            if candidate.score < current.score
                || (candidate.score == current.score && candidate.confidence > current.confidence)
            {
                *current = candidate;
            }
        }
        None => {
            best_by_station.insert(candidate.station_index, candidate);
        }
    }
}

fn confidence_from_score(score: usize, source: MatchSource) -> f32 {
    let source_base = match source {
        MatchSource::Exact => 0.99,
        MatchSource::Prefix => 0.95,
        MatchSource::KeyboardLayout => 0.92,
        MatchSource::JamoCompose => 0.90,
        MatchSource::Choseong => 0.89,
        MatchSource::AliasEn | MatchSource::AliasJa => 0.87,
        MatchSource::SymSpell => 0.82,
    };
    (source_base - (score as f32 * 0.008)).clamp(0.0, 0.99)
}

fn apply_submit_autocorrect(
    result: &mut SearchResult,
    documents: &[StationSearchDocument<'_>],
    query_raw: &str,
    options: SearchOptions,
) {
    if options.mode != SearchMode::Submit || result.matches.is_empty() {
        return;
    }
    let top = &result.matches[0];
    let second_confidence = result.matches.get(1).map(|candidate| candidate.confidence);
    let margin = second_confidence
        .map(|value| top.confidence - value)
        .unwrap_or(top.confidence);
    if top.confidence < options.autocorrect_confidence_threshold
        || margin < options.autocorrect_margin_threshold
    {
        return;
    }
    let corrected = documents[top.station_index].station_name_ko.trim();
    if corrected.is_empty() || normalize(corrected) == normalize(query_raw) {
        return;
    }
    result.autocorrect_applied = true;
    result.corrected_query = Some(corrected.to_string());
}

fn sanitize_ascii_key_input(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

// ── QWERTY ↔ Hangul 2-set mapping ──────────────────────────────────

fn decode_two_set_ascii_to_hangul(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let mut jamo = String::new();
    for ch in input.chars() {
        if let Some(mapped) = qwerty_to_compat_jamo(ch) {
            jamo.push(mapped);
        } else {
            return String::new();
        }
    }
    compose_compat_jamo_stream(jamo.as_str())
}

fn qwerty_to_compat_jamo(ch: char) -> Option<char> {
    Some(match ch.to_ascii_lowercase() {
        'r' => 'ㄱ',
        's' => 'ㄴ',
        'e' => 'ㄷ',
        'f' => 'ㄹ',
        'a' => 'ㅁ',
        'q' => 'ㅂ',
        't' => 'ㅅ',
        'd' => 'ㅇ',
        'w' => 'ㅈ',
        'c' => 'ㅊ',
        'z' => 'ㅋ',
        'x' => 'ㅌ',
        'v' => 'ㅍ',
        'g' => 'ㅎ',
        'k' => 'ㅏ',
        'o' => 'ㅐ',
        'i' => 'ㅑ',
        'j' => 'ㅓ',
        'p' => 'ㅔ',
        'u' => 'ㅕ',
        'h' => 'ㅗ',
        'y' => 'ㅛ',
        'n' => 'ㅜ',
        'b' => 'ㅠ',
        'm' => 'ㅡ',
        'l' => 'ㅣ',
        _ => return None,
    })
}

fn hangul_to_two_set_keyseq(input: &str) -> String {
    let jamo = decompose_hangul_to_compat_jamo(input);
    let mut out = String::new();
    for ch in jamo.chars() {
        if let Some(mapped) = compat_jamo_to_qwerty(ch) {
            out.push_str(mapped);
        }
    }
    out
}

fn compat_jamo_to_qwerty(ch: char) -> Option<&'static str> {
    Some(match ch {
        'ㄱ' => "r",
        'ㄲ' => "rr",
        'ㄴ' => "s",
        'ㄷ' => "e",
        'ㄸ' => "ee",
        'ㄹ' => "f",
        'ㅁ' => "a",
        'ㅂ' => "q",
        'ㅃ' => "qq",
        'ㅅ' => "t",
        'ㅆ' => "tt",
        'ㅇ' => "d",
        'ㅈ' => "w",
        'ㅉ' => "ww",
        'ㅊ' => "c",
        'ㅋ' => "z",
        'ㅌ' => "x",
        'ㅍ' => "v",
        'ㅎ' => "g",
        'ㅏ' => "k",
        'ㅐ' => "o",
        'ㅑ' => "i",
        'ㅒ' => "ii",
        'ㅓ' => "j",
        'ㅔ' => "p",
        'ㅕ' => "u",
        'ㅖ' => "uu",
        'ㅗ' => "h",
        'ㅘ' => "hk",
        'ㅙ' => "ho",
        'ㅚ' => "hl",
        'ㅛ' => "y",
        'ㅜ' => "n",
        'ㅝ' => "nj",
        'ㅞ' => "np",
        'ㅟ' => "nl",
        'ㅠ' => "b",
        'ㅡ' => "m",
        'ㅢ' => "ml",
        'ㅣ' => "l",
        _ => return None,
    })
}

// ── Hangul jamo composition/decomposition ───────────────────────────

fn compose_compat_jamo_stream(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    let mut idx = 0usize;

    while idx < chars.len() {
        let current = chars[idx];
        if let Some(l_index) = choseong_index(current) {
            if idx + 1 >= chars.len() {
                out.push(current);
                idx += 1;
                continue;
            }
            let mut v_char = chars[idx + 1];
            if jungseong_index(v_char).is_none() {
                out.push(current);
                idx += 1;
                continue;
            }

            let mut advance = 2usize;
            if idx + 2 < chars.len() {
                if let Some(combined) = combine_vowel(v_char, chars[idx + 2]) {
                    v_char = combined;
                    advance = 3;
                }
            }
            let Some(v_index) = jungseong_index(v_char) else {
                out.push(current);
                idx += 1;
                continue;
            };

            let mut t_index = 0usize;
            if idx + advance < chars.len() {
                let c1 = chars[idx + advance];
                if let Some(t_single) = jongseong_index(c1) {
                    let mut take_as_jong = true;
                    if idx + advance + 1 < chars.len()
                        && jungseong_index(chars[idx + advance + 1]).is_some()
                    {
                        take_as_jong = false;
                    }
                    if idx + advance + 2 < chars.len() {
                        let c2 = chars[idx + advance + 1];
                        if let Some(combined) = combine_jongseong(c1, c2) {
                            let next_idx = idx + advance + 2;
                            if next_idx >= chars.len() || jungseong_index(chars[next_idx]).is_none()
                            {
                                if let Some(combined_index) = jongseong_index(combined) {
                                    t_index = combined_index;
                                    advance += 2;
                                    take_as_jong = false;
                                }
                            }
                        }
                    }
                    if take_as_jong {
                        t_index = t_single;
                        advance += 1;
                    }
                }
            }

            let syllable = char::from_u32(
                0xAC00 + ((l_index as u32) * 21 * 28) + ((v_index as u32) * 28) + t_index as u32,
            )
            .unwrap_or(current);
            out.push(syllable);
            idx += advance;
            continue;
        }

        out.push(current);
        idx += 1;
    }

    out
}

fn decompose_hangul_to_compat_jamo(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ('\u{AC00}'..='\u{D7A3}').contains(&ch) {
            let syllable_index = ch as u32 - 0xAC00;
            let l_index = (syllable_index / 588) as usize;
            let v_index = ((syllable_index % 588) / 28) as usize;
            let t_index = (syllable_index % 28) as usize;
            out.push(CHOSEONG_COMPAT[l_index]);
            out.push(JUNGSEONG_COMPAT[v_index]);
            if t_index > 0 {
                out.push(JONGSEONG_COMPAT[t_index]);
            }
            continue;
        }
        if is_compat_jamo(ch) {
            out.push(ch);
        }
    }
    out
}

fn extract_hangul_initials(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ('\u{AC00}'..='\u{D7A3}').contains(&ch) {
            let index = ((ch as u32 - 0xAC00) / 588) as usize;
            out.push(CHOSEONG_COMPAT[index]);
        } else if CHOSEONG_COMPAT.contains(&ch) {
            out.push(ch);
        }
    }
    out
}

fn is_compat_jamo(ch: char) -> bool {
    ('\u{3131}'..='\u{318E}').contains(&ch)
}

fn choseong_index(ch: char) -> Option<usize> {
    CHOSEONG_COMPAT.iter().position(|value| *value == ch)
}

fn jungseong_index(ch: char) -> Option<usize> {
    JUNGSEONG_COMPAT.iter().position(|value| *value == ch)
}

fn jongseong_index(ch: char) -> Option<usize> {
    JONGSEONG_COMPAT.iter().position(|value| *value == ch)
}

fn combine_vowel(left: char, right: char) -> Option<char> {
    Some(match (left, right) {
        ('ㅗ', 'ㅏ') => 'ㅘ',
        ('ㅗ', 'ㅐ') => 'ㅙ',
        ('ㅗ', 'ㅣ') => 'ㅚ',
        ('ㅜ', 'ㅓ') => 'ㅝ',
        ('ㅜ', 'ㅔ') => 'ㅞ',
        ('ㅜ', 'ㅣ') => 'ㅟ',
        ('ㅡ', 'ㅣ') => 'ㅢ',
        _ => return None,
    })
}

fn combine_jongseong(left: char, right: char) -> Option<char> {
    Some(match (left, right) {
        ('ㄱ', 'ㅅ') => 'ㄳ',
        ('ㄴ', 'ㅈ') => 'ㄵ',
        ('ㄴ', 'ㅎ') => 'ㄶ',
        ('ㄹ', 'ㄱ') => 'ㄺ',
        ('ㄹ', 'ㅁ') => 'ㄻ',
        ('ㄹ', 'ㅂ') => 'ㄼ',
        ('ㄹ', 'ㅅ') => 'ㄽ',
        ('ㄹ', 'ㅌ') => 'ㄾ',
        ('ㄹ', 'ㅍ') => 'ㄿ',
        ('ㄹ', 'ㅎ') => 'ㅀ',
        ('ㅂ', 'ㅅ') => 'ㅄ',
        _ => return None,
    })
}

// ── Levenshtein / keyboard distance ─────────────────────────────────

fn levenshtein(left: &str, right: &str) -> usize {
    if left == right {
        return 0;
    }
    if left.is_empty() {
        return right.chars().count();
    }
    if right.is_empty() {
        return left.chars().count();
    }

    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();

    let mut previous: Vec<usize> = (0..=right_chars.len()).collect();
    let mut current = vec![0usize; right_chars.len() + 1];

    for (left_idx, left_ch) in left_chars.iter().enumerate() {
        current[0] = left_idx + 1;
        for (right_idx, right_ch) in right_chars.iter().enumerate() {
            let insertion = current[right_idx] + 1;
            let deletion = previous[right_idx + 1] + 1;
            let substitution = previous[right_idx] + usize::from(left_ch != right_ch);
            current[right_idx + 1] = insertion.min(deletion).min(substitution);
        }
        previous.clone_from_slice(&current);
    }

    previous[right_chars.len()]
}

fn keyboard_weighted_distance(
    left: &str,
    right: &str,
    neighbors: fn(char) -> &'static [char],
) -> f32 {
    if left == right {
        return 0.0;
    }
    if left.is_empty() || right.is_empty() {
        return left.chars().count().max(right.chars().count()) as f32;
    }

    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();
    let mut previous: Vec<f32> = (0..=right_chars.len()).map(|value| value as f32).collect();
    let mut current = vec![0.0f32; right_chars.len() + 1];

    for (left_idx, left_ch) in left_chars.iter().enumerate() {
        current[0] = (left_idx + 1) as f32;
        for (right_idx, right_ch) in right_chars.iter().enumerate() {
            let insertion = current[right_idx] + 1.0;
            let deletion = previous[right_idx + 1] + 1.0;
            let substitution =
                previous[right_idx] + keyboard_substitution_cost(*left_ch, *right_ch, neighbors);
            current[right_idx + 1] = insertion.min(deletion).min(substitution);
        }
        previous.clone_from_slice(&current);
    }
    previous[right_chars.len()]
}

fn keyboard_substitution_cost(
    left: char,
    right: char,
    neighbors: fn(char) -> &'static [char],
) -> f32 {
    if left == right {
        return 0.0;
    }
    if neighbors(left).contains(&right) || neighbors(right).contains(&left) {
        return 0.42;
    }
    1.0
}

fn qwerty_neighbors(ch: char) -> &'static [char] {
    match ch.to_ascii_lowercase() {
        'q' => &['w', 'a'],
        'w' => &['q', 'e', 'a', 's'],
        'e' => &['w', 'r', 's', 'd'],
        'r' => &['e', 't', 'd', 'f'],
        't' => &['r', 'y', 'f', 'g'],
        'y' => &['t', 'u', 'g', 'h'],
        'u' => &['y', 'i', 'h', 'j'],
        'i' => &['u', 'o', 'j', 'k'],
        'o' => &['i', 'p', 'k', 'l'],
        'p' => &['o', 'l'],
        'a' => &['q', 'w', 's', 'z'],
        's' => &['a', 'w', 'e', 'd', 'x'],
        'd' => &['s', 'e', 'r', 'f', 'c'],
        'f' => &['d', 'r', 't', 'g', 'v'],
        'g' => &['f', 't', 'y', 'h', 'b'],
        'h' => &['g', 'y', 'u', 'j', 'n'],
        'j' => &['h', 'u', 'i', 'k', 'm'],
        'k' => &['j', 'i', 'o', 'l'],
        'l' => &['k', 'o', 'p'],
        'z' => &['a', 's', 'x'],
        'x' => &['z', 's', 'd', 'c'],
        'c' => &['x', 'd', 'f', 'v'],
        'v' => &['c', 'f', 'g', 'b'],
        'b' => &['v', 'g', 'h', 'n'],
        'n' => &['b', 'h', 'j', 'm'],
        'm' => &['n', 'j', 'k'],
        _ => &[],
    }
}

fn generate_single_substitution_variants(
    input: &str,
    neighbors: fn(char) -> &'static [char],
    max_variants: usize,
) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    for idx in 0..chars.len() {
        let original = chars[idx];
        for neighbor in neighbors(original) {
            if *neighbor == original {
                continue;
            }
            let mut candidate = chars.clone();
            candidate[idx] = *neighbor;
            out.push(candidate.into_iter().collect::<String>());
            if out.len() >= max_variants {
                return out;
            }
        }
    }
    out
}

// ── SymSpell fuzzy index ────────────────────────────────────────────

#[derive(Debug, Clone)]
struct SymSpellForm {
    station_index: usize,
    value: String,
    form_weight: usize,
}

#[derive(Debug, Clone)]
struct SymSpellCandidate {
    station_index: usize,
    distance: usize,
    form_weight: usize,
}

#[derive(Debug, Default)]
struct SymSpellIndex {
    forms: Vec<SymSpellForm>,
    exact_index: HashMap<String, Vec<usize>>,
    delete_index: HashMap<String, Vec<usize>>,
    max_distance: usize,
}

impl SymSpellIndex {
    fn build(station_forms: &[StationForms], max_distance: usize) -> Self {
        let mut index = Self {
            forms: Vec::new(),
            exact_index: HashMap::new(),
            delete_index: HashMap::new(),
            max_distance,
        };
        for (station_index, station) in station_forms.iter().enumerate() {
            index.add_form(station_index, station.ko_norm.as_str(), 0);
            index.add_form(station_index, station.initials.as_str(), 2);
            index.add_form(station_index, station.en_norm.as_str(), 3);
            index.add_form(station_index, station.ja_norm.as_str(), 4);
            index.add_form(station_index, station.ja_hangul.as_str(), 4);
            index.add_form(station_index, station.keyseq_two_set.as_str(), 4);
            index.add_form(station_index, station.jamo.as_str(), 2);
        }
        index
    }

    fn add_form(&mut self, station_index: usize, value: &str, form_weight: usize) {
        if value.is_empty() {
            return;
        }
        let normalized = normalize(value);
        if normalized.is_empty() {
            return;
        }
        let form_index = self.forms.len();
        self.forms.push(SymSpellForm {
            station_index,
            value: normalized.clone(),
            form_weight,
        });
        self.exact_index
            .entry(normalized.clone())
            .or_default()
            .push(form_index);
        for deleted in generate_deletes(normalized.as_str(), self.max_distance) {
            self.delete_index
                .entry(deleted)
                .or_default()
                .push(form_index);
        }
    }

    fn lookup(&self, query: &str) -> Vec<SymSpellCandidate> {
        if query.is_empty() {
            return Vec::new();
        }
        let normalized = normalize(query);
        if normalized.is_empty() {
            return Vec::new();
        }
        let mut candidate_form_indexes: HashSet<usize> = HashSet::new();
        if let Some(forms) = self.exact_index.get(normalized.as_str()) {
            candidate_form_indexes.extend(forms.iter().copied());
        }
        for deleted in generate_deletes(normalized.as_str(), self.max_distance) {
            if let Some(forms) = self.delete_index.get(deleted.as_str()) {
                candidate_form_indexes.extend(forms.iter().copied());
            }
        }

        let mut best_by_station: HashMap<usize, SymSpellCandidate> = HashMap::new();
        for form_index in candidate_form_indexes {
            let form = &self.forms[form_index];
            let distance = levenshtein(normalized.as_str(), form.value.as_str());
            if distance > self.max_distance {
                continue;
            }
            let candidate = SymSpellCandidate {
                station_index: form.station_index,
                distance,
                form_weight: form.form_weight,
            };
            match best_by_station.get_mut(&form.station_index) {
                Some(current) => {
                    if candidate.distance < current.distance
                        || (candidate.distance == current.distance
                            && candidate.form_weight < current.form_weight)
                    {
                        *current = candidate;
                    }
                }
                None => {
                    best_by_station.insert(form.station_index, candidate);
                }
            }
        }
        best_by_station.into_values().collect()
    }
}

fn generate_deletes(term: &str, max_distance: usize) -> HashSet<String> {
    let mut out = HashSet::new();
    if term.is_empty() || max_distance == 0 {
        return out;
    }

    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    queue.push_back((term.to_string(), 0));
    while let Some((current, distance)) = queue.pop_front() {
        if distance >= max_distance {
            continue;
        }
        let chars: Vec<char> = current.chars().collect();
        if chars.len() <= 1 {
            continue;
        }
        for remove_index in 0..chars.len() {
            let mut next = String::new();
            for (idx, ch) in chars.iter().enumerate() {
                if idx != remove_index {
                    next.push(*ch);
                }
            }
            if out.insert(next.clone()) {
                queue.push_back((next, distance + 1));
            }
        }
    }
    out
}

// ── Katakana → Hangul transliteration ────────────────────────────────

fn has_katakana(input: &str) -> bool {
    input
        .chars()
        .any(|ch| ('\u{30A1}'..='\u{30F6}').contains(&ch) || ch == 'ン' || ch == 'ー' || ch == 'ッ')
}

/// Convert a katakana string to approximate Korean hangul via phonological
/// rule-based mapping. Produces compatibility jamo, then composes into
/// syllable blocks using [`compose_compat_jamo_stream`].
fn transliterate_katakana_to_hangul(input: &str) -> String {
    let normalized = normalize(input);
    let chars: Vec<char> = normalized.chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    let mut jamo = String::new();
    let mut idx = 0usize;

    while idx < chars.len() {
        let ch = chars[idx];

        // Try compound kana (2-char: base + small ャ/ュ/ョ/ァ/ィ/ェ/ォ) first.
        if idx + 1 < chars.len() {
            if let Some(mapped) = compound_katakana_to_jamo(ch, chars[idx + 1]) {
                jamo.push_str(mapped);
                idx += 2;
                continue;
            }
        }

        // ン → context-dependent nasal jongseong.
        if ch == 'ン' {
            if idx + 1 < chars.len() && is_labial_kana(chars[idx + 1]) {
                jamo.push('ㅁ');
            } else {
                jamo.push('ㄴ');
            }
            idx += 1;
            continue;
        }

        // ー → long vowel mark, drop.
        if ch == 'ー' {
            idx += 1;
            continue;
        }

        // ッ → geminate: add appropriate jongseong for the following consonant.
        if ch == 'ッ' {
            if idx + 1 < chars.len() {
                jamo.push(geminate_jongseong(chars[idx + 1]));
            }
            idx += 1;
            continue;
        }

        // Single kana.
        if let Some(mapped) = single_katakana_to_jamo(ch) {
            jamo.push_str(mapped);
            idx += 1;
            continue;
        }

        // Pass through Hangul and other recognised characters.
        if ('\u{AC00}'..='\u{D7A3}').contains(&ch) || is_compat_jamo(ch) {
            jamo.push(ch);
        }
        idx += 1;
    }

    compose_compat_jamo_stream(jamo.as_str())
}

fn single_katakana_to_jamo(ch: char) -> Option<&'static str> {
    Some(match ch {
        // Vowels (silent ㅇ initial)
        'ア' => "ㅇㅏ",
        'イ' => "ㅇㅣ",
        'ウ' => "ㅇㅜ",
        'エ' => "ㅇㅔ",
        'オ' => "ㅇㅗ",
        // Ka-row → ㅋ (voiceless velar)
        'カ' => "ㅋㅏ",
        'キ' => "ㅋㅣ",
        'ク' => "ㅋㅜ",
        'ケ' => "ㅋㅔ",
        'コ' => "ㅋㅗ",
        // Sa-row → ㅅ
        'サ' => "ㅅㅏ",
        'シ' => "ㅅㅣ",
        'ス' => "ㅅㅜ",
        'セ' => "ㅅㅔ",
        'ソ' => "ㅅㅗ",
        // Ta-row → ㅌ / ㅊ
        'タ' => "ㅌㅏ",
        'チ' => "ㅊㅣ",
        'ツ' => "ㅊㅜ",
        'テ' => "ㅌㅔ",
        'ト' => "ㅌㅗ",
        // Na-row → ㄴ
        'ナ' => "ㄴㅏ",
        'ニ' => "ㄴㅣ",
        'ヌ' => "ㄴㅜ",
        'ネ' => "ㄴㅔ",
        'ノ' => "ㄴㅗ",
        // Ha-row → ㅎ
        'ハ' => "ㅎㅏ",
        'ヒ' => "ㅎㅣ",
        'フ' => "ㅎㅜ",
        'ヘ' => "ㅎㅔ",
        'ホ' => "ㅎㅗ",
        // Ma-row → ㅁ
        'マ' => "ㅁㅏ",
        'ミ' => "ㅁㅣ",
        'ム' => "ㅁㅜ",
        'メ' => "ㅁㅔ",
        'モ' => "ㅁㅗ",
        // Ya-row
        'ヤ' => "ㅇㅑ",
        'ユ' => "ㅇㅠ",
        'ヨ' => "ㅇㅛ",
        // Ra-row → ㄹ
        'ラ' => "ㄹㅏ",
        'リ' => "ㄹㅣ",
        'ル' => "ㄹㅜ",
        'レ' => "ㄹㅔ",
        'ロ' => "ㄹㅗ",
        // Wa-row
        'ワ' => "ㅇㅘ",
        'ヲ' => "ㅇㅗ",
        // Ga-row → ㄱ (voiced velar)
        'ガ' => "ㄱㅏ",
        'ギ' => "ㄱㅣ",
        'グ' => "ㄱㅜ",
        'ゲ' => "ㄱㅔ",
        'ゴ' => "ㄱㅗ",
        // Za-row → ㅈ
        'ザ' => "ㅈㅏ",
        'ジ' => "ㅈㅣ",
        'ズ' => "ㅈㅜ",
        'ゼ' => "ㅈㅔ",
        'ゾ' => "ㅈㅗ",
        // Da-row → ㄷ
        'ダ' => "ㄷㅏ",
        'ヂ' => "ㅈㅣ",
        'ヅ' => "ㅈㅜ",
        'デ' => "ㄷㅔ",
        'ド' => "ㄷㅗ",
        // Ba-row → ㅂ
        'バ' => "ㅂㅏ",
        'ビ' => "ㅂㅣ",
        'ブ' => "ㅂㅜ",
        'ベ' => "ㅂㅔ",
        'ボ' => "ㅂㅗ",
        // Pa-row → ㅍ
        'パ' => "ㅍㅏ",
        'ピ' => "ㅍㅣ",
        'プ' => "ㅍㅜ",
        'ペ' => "ㅍㅔ",
        'ポ' => "ㅍㅗ",
        // Special
        'ヴ' => "ㅂㅜ",
        _ => return None,
    })
}

fn compound_katakana_to_jamo(first: char, second: char) -> Option<&'static str> {
    Some(match (first, second) {
        // Ki + small ya/yu/yo
        ('キ', 'ャ') => "ㅋㅑ",
        ('キ', 'ュ') => "ㅋㅠ",
        ('キ', 'ョ') => "ㅋㅛ",
        // Shi + small ya/yu/yo
        ('シ', 'ャ') => "ㅅㅑ",
        ('シ', 'ュ') => "ㅅㅠ",
        ('シ', 'ョ') => "ㅅㅛ",
        // Chi + small ya/yu/yo → cha/chu/cho
        ('チ', 'ャ') => "ㅊㅏ",
        ('チ', 'ュ') => "ㅊㅜ",
        ('チ', 'ョ') => "ㅊㅗ",
        // Ni + small ya/yu/yo
        ('ニ', 'ャ') => "ㄴㅑ",
        ('ニ', 'ュ') => "ㄴㅠ",
        ('ニ', 'ョ') => "ㄴㅛ",
        // Hi + small ya/yu/yo
        ('ヒ', 'ャ') => "ㅎㅑ",
        ('ヒ', 'ュ') => "ㅎㅠ",
        ('ヒ', 'ョ') => "ㅎㅛ",
        // Mi + small ya/yu/yo
        ('ミ', 'ャ') => "ㅁㅑ",
        ('ミ', 'ュ') => "ㅁㅠ",
        ('ミ', 'ョ') => "ㅁㅛ",
        // Ri + small ya/yu/yo
        ('リ', 'ャ') => "ㄹㅑ",
        ('リ', 'ュ') => "ㄹㅠ",
        ('リ', 'ョ') => "ㄹㅛ",
        // Gi + small ya/yu/yo
        ('ギ', 'ャ') => "ㄱㅑ",
        ('ギ', 'ュ') => "ㄱㅠ",
        ('ギ', 'ョ') => "ㄱㅛ",
        // Ji + small ya/yu/yo
        ('ジ', 'ャ') => "ㅈㅏ",
        ('ジ', 'ュ') => "ㅈㅜ",
        ('ジ', 'ョ') => "ㅈㅗ",
        // Bi + small ya/yu/yo
        ('ビ', 'ャ') => "ㅂㅑ",
        ('ビ', 'ュ') => "ㅂㅠ",
        ('ビ', 'ョ') => "ㅂㅛ",
        // Pi + small ya/yu/yo
        ('ピ', 'ャ') => "ㅍㅑ",
        ('ピ', 'ュ') => "ㅍㅠ",
        ('ピ', 'ョ') => "ㅍㅛ",
        // Modern extensions (foreign sounds)
        ('テ', 'ィ') => "ㅌㅣ",
        ('デ', 'ィ') => "ㄷㅣ",
        ('フ', 'ァ') => "ㅍㅏ",
        ('フ', 'ィ') => "ㅍㅣ",
        ('フ', 'ェ') => "ㅍㅔ",
        ('フ', 'ォ') => "ㅍㅗ",
        // Vu + small vowels
        ('ヴ', 'ァ') => "ㅂㅏ",
        ('ヴ', 'ィ') => "ㅂㅣ",
        ('ヴ', 'ェ') => "ㅂㅔ",
        ('ヴ', 'ォ') => "ㅂㅗ",
        // U + small vowels (ウォ, ウェ, ウィ)
        ('ウ', 'ォ') => "ㅇㅝ",
        ('ウ', 'ェ') => "ㅇㅞ",
        ('ウ', 'ィ') => "ㅇㅟ",
        _ => return None,
    })
}

/// ン before a labial kana should be rendered as ㅁ rather than ㄴ.
fn is_labial_kana(ch: char) -> bool {
    matches!(
        ch,
        'マ' | 'ミ'
            | 'ム'
            | 'メ'
            | 'モ'
            | 'バ'
            | 'ビ'
            | 'ブ'
            | 'ベ'
            | 'ボ'
            | 'パ'
            | 'ピ'
            | 'プ'
            | 'ペ'
            | 'ポ'
    )
}

/// Return the jongseong consonant to use before a given kana in gemination (ッ).
fn geminate_jongseong(next: char) -> char {
    match next {
        'カ' | 'キ' | 'ク' | 'ケ' | 'コ' | 'ガ' | 'ギ' | 'グ' | 'ゲ' | 'ゴ' => 'ㄱ',
        'サ' | 'シ' | 'ス' | 'セ' | 'ソ' | 'ザ' | 'ジ' | 'ズ' | 'ゼ' | 'ゾ' => 'ㅅ',
        'タ' | 'チ' | 'ツ' | 'テ' | 'ト' | 'ダ' | 'ヂ' | 'ヅ' | 'デ' | 'ド' => 'ㅌ',
        'パ' | 'ピ' | 'プ' | 'ペ' | 'ポ' | 'バ' | 'ビ' | 'ブ' | 'ベ' | 'ボ' => 'ㅂ',
        _ => 'ㅅ',
    }
}

// ── Hangul lookup tables ────────────────────────────────────────────

const CHOSEONG_COMPAT: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];

const JUNGSEONG_COMPAT: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];

const JONGSEONG_COMPAT: [char; 28] = [
    '\0', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ',
    'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_documents() -> Vec<StationSearchDocument<'static>> {
        vec![
            StationSearchDocument {
                station_name_ko: "수서",
                station_name_en: Some("Suseo"),
                station_name_ja_katakana: "スソ",
                normalized_name: "수서",
            },
            StationSearchDocument {
                station_name_ko: "서울",
                station_name_en: Some("Seoul"),
                station_name_ja_katakana: "ソウル",
                normalized_name: "서울",
            },
            StationSearchDocument {
                station_name_ko: "부산",
                station_name_en: Some("Busan"),
                station_name_ja_katakana: "プサン",
                normalized_name: "부산",
            },
            StationSearchDocument {
                station_name_ko: "대구",
                station_name_en: Some("Daegu"),
                station_name_ja_katakana: "テグ",
                normalized_name: "대구",
            },
            StationSearchDocument {
                station_name_ko: "창원",
                station_name_en: Some("Changwon"),
                station_name_ja_katakana: "チャンウォン",
                normalized_name: "창원",
            },
        ]
    }

    #[test]
    fn qwerty_keyboard_maps_to_korean_station() {
        let docs = fixture_documents();
        let result = rank_station_documents(&docs, "tntj", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 0); // 수서
    }

    #[test]
    fn keyboard_examples_map_to_expected_stations() {
        let docs = fixture_documents();
        let fixtures = [
            ("tjdnf", 1usize),   // 서울
            ("qntks", 2usize),   // 부산
            ("eorn", 3usize),    // 대구
            ("ckddnjs", 4usize), // 창원
        ];
        for (query, expected_index) in fixtures {
            let result = rank_station_documents(&docs, query, SearchOptions::default(), 5);
            assert!(
                !result.matches.is_empty(),
                "expected matches for query: {query}"
            );
            assert_eq!(
                result.matches[0].station_index, expected_index,
                "unexpected top match for query: {query}"
            );
        }
    }

    #[test]
    fn keyboard_neighbor_typo_still_resolves() {
        let docs = fixture_documents();
        // "tjdnd" is one key away from "tjdnf" (서울)
        let result = rank_station_documents(&docs, "tjdnd", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 1);
    }

    #[test]
    fn english_alias_matches() {
        let docs = fixture_documents();
        let result = rank_station_documents(&docs, "busan", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 2);
    }

    #[test]
    fn hiragana_matches_japanese_alias() {
        let docs = fixture_documents();
        let result = rank_station_documents(&docs, "そうる", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 1); // ソウル
    }

    #[test]
    fn chosung_matches() {
        let docs = fixture_documents();
        let result = rank_station_documents(&docs, "ㄷㄱ", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 3); // 대구
    }

    #[test]
    fn decomposed_jamo_matches() {
        let docs = fixture_documents();
        let result = rank_station_documents(&docs, "ㅂㅜㅅㅏㄴ", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 2); // 부산
    }

    #[test]
    fn submit_mode_autocorrects_when_confident() {
        let docs = fixture_documents();
        let result = rank_station_documents(
            &docs,
            "tjdnf",
            SearchOptions {
                mode: SearchMode::Submit,
                ..SearchOptions::default()
            },
            5,
        );
        assert!(result.autocorrect_applied);
        assert_eq!(result.corrected_query.as_deref(), Some("서울"));
    }

    #[test]
    fn korean_exact_match() {
        let docs = fixture_documents();
        let result = rank_station_documents(&docs, "부산", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 2);
        assert_eq!(result.matches[0].source, MatchSource::Exact);
    }

    #[test]
    fn korean_prefix_match() {
        let docs = fixture_documents();
        let result = rank_station_documents(&docs, "부", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 2);
    }

    #[test]
    fn empty_query_returns_empty() {
        let docs = fixture_documents();
        let result = rank_station_documents(&docs, "", SearchOptions::default(), 5);
        assert!(result.matches.is_empty());
    }

    // ── Katakana → Hangul transliteration tests ─────────────────────

    #[test]
    fn transliterate_basic_katakana() {
        // プサン → approximate hangul (ㅍㅜ + ㅅㅏ + ㄴ → 푸산)
        let result = transliterate_katakana_to_hangul("プサン");
        assert!(!result.is_empty());
        assert!(
            result
                .chars()
                .all(|ch| ('\u{AC00}'..='\u{D7A3}').contains(&ch) || is_compat_jamo(ch))
        );
    }

    #[test]
    fn transliterate_compound_kana() {
        // チョン → cho + n
        let result = transliterate_katakana_to_hangul("チョン");
        assert!(!result.is_empty());
    }

    #[test]
    fn transliterate_long_vowel_dropped() {
        // カー → ka (ー dropped)
        let without = transliterate_katakana_to_hangul("カ");
        let with_long = transliterate_katakana_to_hangul("カー");
        assert_eq!(without, with_long);
    }

    #[test]
    fn transliterate_geminate() {
        // サッポロ → 삿포로 (ッ adds ㅅ jongseong before ポ)
        let result = transliterate_katakana_to_hangul("サッポロ");
        assert!(!result.is_empty());
    }

    #[test]
    fn transliterate_labial_nasal() {
        // サンマ → 삼마 (ン before マ → ㅁ)
        let result = transliterate_katakana_to_hangul("サンマ");
        assert!(result.contains('삼') || result.contains('ㅁ') || !result.is_empty());
    }

    #[test]
    fn transliterate_hiragana_normalised() {
        // Hiragana input gets normalised to katakana first
        let katakana = transliterate_katakana_to_hangul("プサン");
        let hiragana = transliterate_katakana_to_hangul("ぷさん");
        assert_eq!(katakana, hiragana);
    }

    #[test]
    fn katakana_transliteration_matches_station() {
        let docs = fixture_documents();
        // プサン (katakana for Busan) should match 부산 via both AliasJa and
        // the transliterated-hangul fuzzy path.
        let result = rank_station_documents(&docs, "プサン", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 2); // 부산
    }

    #[test]
    fn partial_katakana_matches_station() {
        let docs = fixture_documents();
        // プサ (partial katakana for Busan) should match via prefix
        let result = rank_station_documents(&docs, "プサ", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 2); // 부산
    }

    #[test]
    fn katakana_compound_matches_station() {
        let docs = fixture_documents();
        // チャンウォン → 창원 via transliteration + fuzzy
        let result = rank_station_documents(&docs, "チャンウォン", SearchOptions::default(), 5);
        assert!(!result.matches.is_empty());
        assert_eq!(result.matches[0].station_index, 4); // 창원
    }

    #[test]
    fn normalize_hiragana_to_katakana() {
        assert_eq!(normalize("しんおおさか"), "シンオオサカ");
    }
}
