//! # Yekdast: یک کتابخانه قابل تنظیم برای یکسان‌سازی متن فارسی
//!
//! `yekdast` یک ابزار جامع برای نرمال‌سازی و استانداردسازی متن فارسی (Farsi) در زبان Rust است.
//! این کتابخانه به کاربران اجازه می‌دهد تا دیکشنری‌های سفارشی خود را برای نرمال‌سازی کلمات محاوره‌ای
//! و مدیریت کلمات مرکب ارائه دهند که آن را بسیار انعطاف‌پذیر می‌کند.

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use regex::Regex;

// --- ساختارهای عمومی برای تنظیمات ---

/// سیاست مدیریت ارقام را مشخص می‌کند.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DigitPolicy {
    #[serde(rename = "fa")] Fa,       // تبدیل به ارقام فارسی
    #[serde(rename = "ar")] Ar,       // تبدیل به ارقام عربی
    #[serde(rename = "latin")] Latin, // تبدیل به ارقام لاتین
    #[serde(rename = "auto")] Auto,     // تشخیص خودکار بر اساس متن
    #[serde(rename = "preserve")] Preserve, // بدون تغییر
}

/// سیاست مدیریت علائم نگارشی را مشخص می‌کند.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PunctPolicy {
    #[serde(rename = "fa")] Fa,     // تبدیل به علائم فارسی (مانند ، ؛ ؟)
    #[serde(rename = "latin")] Latin, // تبدیل به علائم لاتین
    #[serde(rename = "keep")] Keep,   // بدون تغییر
}

/// سیاست اعمال نیم‌فاصله (ZWNJ) را مشخص می‌کند.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZwnjPolicy {
    #[serde(rename = "smart")] Smart, // اعمال هوشمند برای پیشوندها، پسوندها و کلمات مرکب
    #[serde(rename = "force")] Force, // اجبار نیم‌فاصله بین هر دو حرف فارسی
    #[serde(rename = "none")] None,   // عدم اعمال نیم‌فاصله
}

/// فرمت نرمال‌سازی یونیکد را مشخص می‌کند.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnicodeForm {
    #[serde(rename = "NFC")] Nfc,
    #[serde(rename = "NFKC")] Nfkc, // حالت پیشنهادی
}

/// ساختار اصلی برای نگهداری تمام تنظیمات نرمال‌سازی.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NormalizeOptions {
    pub locale: String,
    pub unicode_form: UnicodeForm,
    pub remove_diacritics: bool,
    pub remove_tatweel: bool,
    pub digits: DigitPolicy,
    pub punctuation: PunctPolicy,
    pub zwnj: ZwnjPolicy,
    pub trim: bool,
    pub squeeze_whitespace: bool,
    pub normalize_newlines: bool,
    pub drop_bidi_controls: bool,
    pub confusable_safe: bool,
    pub protect_urls: bool,
    pub protect_emails: bool,
    pub protect_code: bool,
    pub protect_html_tags: bool,
    
    /// یک دیکشنری سفارشی برای تبدیل کلمات محاوره‌ای به رسمی.
    /// کلید، کلمه محاوره‌ای و مقدار، معادل رسمی آن است.
    pub slang_map: HashMap<String, String>,

    /// لیستی از کلمات مرکب که باید با نیم‌فاصله به هم بچسبند.
    /// هر آیتم باید یک رشته با کلمات جدا شده با فاصله باشد، مانند "کتاب خانه".
    pub zwnj_compound_words: Vec<String>,
    
    /// لیستی از قوانین جایگزینی سفارشی `(از, به)`. این قوانین در مراحل اولیه اعمال می‌شوند.
    pub custom_rules: Vec<(String, String)>,
}

/// پیاده‌سازی مقادیر پیش‌فرض برای تنظیمات.
impl Default for NormalizeOptions {
    fn default() -> Self {
        NormalizeOptions {
            locale: "fa-IR".to_string(),
            unicode_form: UnicodeForm::Nfkc,
            remove_diacritics: true,
            remove_tatweel: true,
            digits: DigitPolicy::Auto,
            punctuation: PunctPolicy::Fa,
            zwnj: ZwnjPolicy::Smart,
            trim: true,
            squeeze_whitespace: true,
            normalize_newlines: true,
            drop_bidi_controls: false,
            confusable_safe: false,
            protect_urls: true,
            protect_emails: true,
            protect_code: true,
            protect_html_tags: true,
            slang_map: HashMap::new(),
            zwnj_compound_words: Vec::new(),
            custom_rules: Vec::new(),
        }
    }
}

// --- داده‌های ایستا و عبارات باقاعده ---
// این داده‌ها و Regexها به صورت Lazy کامپایل می‌شوند تا در اولین استفاده آماده شوند و عملکرد را بهینه کنند.
static ARABIC_TO_PERSIAN: Lazy<HashMap<char, char>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('ي', 'ی'); map.insert('ى', 'ی');
    map.insert('إ', 'ا'); map.insert('أ', 'ا'); map.insert('ٱ', 'ا');
    map.insert('ؤ', 'و');
    map.insert('ك', 'ک'); map.insert('ﻙ', 'ک');
    map.insert('ۀ', 'ه'); map.insert('ة', 'ه');
    map
});

static LATIN_DIGITS: Lazy<Vec<char>> = Lazy::new(|| "0123456789".chars().collect());
static PERSIAN_DIGITS: Lazy<Vec<char>> = Lazy::new(|| "۰۱۲۳۴۵۶۷۸۹".chars().collect());
static ARABIC_DIGITS: Lazy<Vec<char>> = Lazy::new(|| "٠١٢٣٤٥٦٧٨٩".chars().collect());

static PUNCT_FA: Lazy<HashMap<char, char>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(',', '،'); map.insert(';', '؛'); map.insert('?', '؟');
    map
});

static PUNCT_LATIN: Lazy<HashMap<char, char>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('،', ','); map.insert('؛', ';'); map.insert('؟', '?');
    map
});

static CONFUSABLES_MAP: Lazy<HashMap<char, char>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert('۰', '0'); map.insert('۱', '1'); map.insert('۲', '2'); map.insert('۳', '3');
    map.insert('۴', '4'); map.insert('۵', '5'); map.insert('۶', '6'); map.insert('۷', '7');
    map.insert('۸', '8'); map.insert('۹', '9');
    map.insert('٠', '0'); map.insert('١', '1'); map.insert('٢', '2'); map.insert('٣', '3');
    map.insert('٤', '4'); map.insert('٥', '5'); map.insert('٦', '6'); map.insert('٧', '7');
    map.insert('٨', '8'); map.insert('٩', '9');
    map.insert('ك', 'ک'); map.insert('ي', 'ی'); map.insert('ى', 'ی'); map.insert('ة', 'ه');
    map.insert('ۀ', 'ه');
    map
});

// Regexهای ثابت برای قوانین عمومی
static MI_NEMI_PREFIX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(می|نمی)\s+([\u0600-\u06FF]{2,})").unwrap());
static SUFFIXES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\u0600-\u06FF]{2,})\s+(ها|های|تر|ترین|ام|ات|اش|ایم|اید|اند|گان|مند|وار)\b").unwrap());
static FORCE_ZWNJ_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([آ-ی])\s+([آ-ی])").unwrap());
static SQUEEZE_WHITESPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t\u{00A0}]+").unwrap());
static SQUEEZE_POST_PUNCT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([،,؛;:!?.)\]}])").unwrap());
static SQUEEZE_PRE_PUNCT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\(«\[{])\s+").unwrap());

// Regex برای محافظت از بخش‌های خاص متن
static URL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\bhttps?:\/\/[^\s)>"']+|www\.[^\s)>"']+"#).unwrap());
static EMAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b").unwrap());
static CODE_BLOCK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"```[\s\S]*?```").unwrap());
static INLINE_CODE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`[^`]*`").unwrap());
static HTML_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<\/?[\w\d-]+(\s+[^>]*?)?>").unwrap());


// --- توابع کمکی خصوصی ---

/// اعمال نرمال‌سازی یونیکد
fn to_form(s: &str, form: &UnicodeForm) -> String {
    match form {
        UnicodeForm::Nfc => s.nfc().collect(),
        UnicodeForm::Nfkc => s.nfkc().collect(),
    }
}

/// حذف کاراکتر کشیدگی (تطویل)
fn strip_tatweel(s: &str) -> String { s.replace('\u{0640}', "") }

/// حذف اعراب
fn strip_diacritics(s: &str) -> String {
    s.chars().filter(|&c| !matches!(c, '\u{0610}'..='\u{061A}' | '\u{064B}'..='\u{065F}' | '\u{06D6}'..='\u{06ED}')).collect()
}

/// حذف کاراکترهای کنترل جهت متن
fn drop_bidi_controls(s: &str) -> String {
    s.chars().filter(|&c| !matches!(c, '\u{200E}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')).collect()
}

/// تبدیل حروف عربی به معادل فارسی
fn unify_letters_fa(s: &str) -> String {
    s.chars().map(|c| *ARABIC_TO_PERSIAN.get(&c).unwrap_or(&c)).collect()
}

/// تابع اصلی برای نرمال‌سازی ارقام
fn normalize_digits_impl(s: &str, target_digits: &[char]) -> String {
    s.chars().map(|c| {
        if let Some(idx) = LATIN_DIGITS.iter().position(|&d| d == c) { target_digits[idx] } 
        else if let Some(idx) = PERSIAN_DIGITS.iter().position(|&d| d == c) { target_digits[idx] } 
        else if let Some(idx) = ARABIC_DIGITS.iter().position(|&d| d == c) { target_digits[idx] } 
        else { c }
    }).collect()
}

/// نرمال‌سازی ارقام بر اساس سیاست تعریف شده
fn normalize_digits(s: &str, policy: &DigitPolicy) -> String {
    match policy {
        DigitPolicy::Preserve => s.to_string(),
        DigitPolicy::Auto => {
            if s.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c)) {
                normalize_digits_impl(s, &PERSIAN_DIGITS)
            } else {
                normalize_digits_impl(s, &LATIN_DIGITS)
            }
        },
        DigitPolicy::Fa => normalize_digits_impl(s, &PERSIAN_DIGITS),
        DigitPolicy::Ar => normalize_digits_impl(s, &ARABIC_DIGITS),
        DigitPolicy::Latin => normalize_digits_impl(s, &LATIN_DIGITS),
    }
}

/// نرمال‌سازی علائم نگارشی بر اساس سیاست تعریف شده
fn normalize_punctuation(s: &str, policy: &PunctPolicy) -> String {
    match policy {
        PunctPolicy::Keep => s.to_string(),
        PunctPolicy::Fa => s.chars().map(|c| *PUNCT_FA.get(&c).unwrap_or(&c)).collect(),
        PunctPolicy::Latin => s.chars().map(|c| *PUNCT_LATIN.get(&c).unwrap_or(&c)).collect(),
    }
}

/// اعمال نرمال‌سازی کلمات محاوره‌ای بر اساس دیکشنری کاربر
fn apply_slang_normalization(s: &str, slang_map: &HashMap<String, String>) -> String {
    if slang_map.is_empty() {
        return s.to_string();
    }
    let slang_keys: Vec<String> = slang_map.keys().map(|k| regex::escape(k)).collect();
    let pattern = format!(r"\b({})\b", slang_keys.join("|"));
    // در صورت خطای Regex، یک الگوی بی‌اثر برمی‌گردانیم تا برنامه خراب نشود
    let slang_re = Regex::new(&pattern).unwrap_or_else(|_| Regex::new("a^").unwrap()); 

    slang_re.replace_all(s, |caps: &regex::Captures| {
        slang_map.get(&caps[0]).cloned().unwrap_or_else(|| caps[0].to_string())
    }).to_string()
}

/// اعمال هوشمند نیم‌فاصله، شامل قوانین ثابت و لیست سفارشی کاربر
fn smart_zwnj(s: &str, zwnj_compound_words: &[String]) -> String {
    let mut s_cloned = s.to_string();
    
    // ۱. اعمال قوانین ثابت برای پیشوندها و پسوندها
    s_cloned = MI_NEMI_PREFIX_RE.replace_all(&s_cloned, "$1\u{200c}$2").to_string();
    s_cloned = SUFFIXES_RE.replace_all(&s_cloned, "$1\u{200c}$2").to_string();

    // ۲. اعمال قوانین سفارشی کاربر برای کلمات مرکب
    if !zwnj_compound_words.is_empty() {
        for compound in zwnj_compound_words {
            if let Some(pos) = compound.find(' ') {
                let (part1, part2) = compound.split_at(pos);
                let with_zwnj = format!("{}\u{200c}{}", part1, part2.trim_start());
                s_cloned = s_cloned.replace(compound, &with_zwnj);
            }
        }
    }

    // ۳. پاک‌سازی فاصله‌های اضافه اطراف نیم‌فاصله
    s_cloned.replace(" \u{200c}", "\u{200c}").replace("\u{200c} ", "\u{200c}")
}

/// اجبار نیم‌فاصله بین تمام حروف فارسی
fn force_zwnj(s: &str) -> String {
    FORCE_ZWNJ_RE.replace_all(s, "$1\u{200c}$2").to_string()
}

/// پاک‌سازی و یکسان‌سازی فاصله‌ها
fn normalize_whitespace(s: &str, squeeze: bool, trim: bool, normalize_newlines: bool) -> String {
    let mut s_cloned = s.to_string();
    if normalize_newlines { s_cloned = s_cloned.replace("\r\n", "\n").replace('\r', "\n"); }
    if squeeze {
        s_cloned = SQUEEZE_WHITESPACE_RE.replace_all(&s_cloned, " ").to_string();
        s_cloned = SQUEEZE_POST_PUNCT_RE.replace_all(&s_cloned, "$1").to_string();
        s_cloned = SQUEEZE_PRE_PUNCT_RE.replace_all(&s_cloned, "$1").to_string();
    }
    if trim { s_cloned = s_cloned.trim().to_string(); }
    s_cloned
}

// --- منطق جداسازی برای محافظت از بخش‌های خاص متن ---
#[derive(Debug, Clone, PartialEq)]
enum SpanKind { Text, Url, Email, Code, Html }
#[derive(Debug, Clone)]
struct Span { kind: SpanKind, start: usize, end: usize }

/// متن را به بخش‌های قابل نرمال‌سازی و بخش‌های محافظت‌شده تقسیم می‌کند.
fn segment(text: &str, options: &NormalizeOptions) -> Vec<Span> {
    let mut blocks: Vec<Span> = vec![Span { kind: SpanKind::Text, start: 0, end: text.len() }];
    let carve = |re: &Regex, kind: SpanKind, blocks: &mut Vec<Span>| {
        let mut new_blocks: Vec<Span> = Vec::new();
        for b in blocks.iter() {
            if b.kind != SpanKind::Text { new_blocks.push(b.clone()); continue; }
            let mut current_pos = b.start;
            for m in re.find_iter(&text[b.start..b.end]) {
                let (s, e) = (b.start + m.start(), b.start + m.end());
                if current_pos < s { new_blocks.push(Span { kind: SpanKind::Text, start: current_pos, end: s }); }
                new_blocks.push(Span { kind: kind.clone(), start: s, end: e });
                current_pos = e;
            }
            if current_pos < b.end { new_blocks.push(Span { kind: SpanKind::Text, start: current_pos, end: b.end }); }
        }
        *blocks = new_blocks;
    };
    if options.protect_html_tags { carve(&HTML_TAG_RE, SpanKind::Html, &mut blocks); }
    if options.protect_code {
        carve(&CODE_BLOCK_RE, SpanKind::Code, &mut blocks);
        carve(&INLINE_CODE_RE, SpanKind::Code, &mut blocks);
    }
    if options.protect_emails { carve(&EMAIL_RE, SpanKind::Email, &mut blocks); }
    if options.protect_urls { carve(&URL_RE, SpanKind::Url, &mut blocks); }
    blocks.sort_by_key(|s| s.start);
    blocks
}

// --- تابع اصلی و عمومی کتابخانه ---
/// یک رشته ورودی را بر اساس تنظیمات داده شده نرمال‌سازی می‌کند.
pub fn normalize_text(input: &str, options: &NormalizeOptions) -> String {
    let mut output = String::new();
    let spans = segment(input, options);
    for span in &spans {
        let chunk = &input[span.start..span.end];
        if span.kind == SpanKind::Text {
            let mut processed_chunk = chunk.to_string();
            processed_chunk = to_form(&processed_chunk, &options.unicode_form);
            if options.remove_tatweel { processed_chunk = strip_tatweel(&processed_chunk); }
            if options.remove_diacritics { processed_chunk = strip_diacritics(&processed_chunk); }
            if options.locale == "fa-IR" { processed_chunk = unify_letters_fa(&processed_chunk); }

            if !options.custom_rules.is_empty() {
                for (from, to) in &options.custom_rules {
                    processed_chunk = processed_chunk.replace(from, to);
                }
            }
            
            processed_chunk = apply_slang_normalization(&processed_chunk, &options.slang_map);

            processed_chunk = normalize_digits(&processed_chunk, &options.digits);
            processed_chunk = normalize_punctuation(&processed_chunk, &options.punctuation);
            
            match &options.zwnj {
                ZwnjPolicy::Smart => processed_chunk = smart_zwnj(&processed_chunk, &options.zwnj_compound_words),
                ZwnjPolicy::Force => processed_chunk = force_zwnj(&processed_chunk),
                ZwnjPolicy::None => (),
            }

            processed_chunk = normalize_whitespace(&processed_chunk, options.squeeze_whitespace, false, options.normalize_newlines);
            if options.drop_bidi_controls { processed_chunk = drop_bidi_controls(&processed_chunk); }
            if options.confusable_safe {
                processed_chunk = processed_chunk.chars().map(|c| *CONFUSABLES_MAP.get(&c).unwrap_or(&c)).collect();
            }
            output.push_str(&processed_chunk);
        } else {
            output.push_str(chunk);
        }
    }
    if options.trim { output.trim().to_string() } else { output }
}

// --- تست‌های واحد ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_normalization() {
        let text = "مي روم 12,345 ريال، كتاب ها ١٢٣";
        let options = NormalizeOptions { digits: DigitPolicy::Fa, punctuation: PunctPolicy::Fa, zwnj: ZwnjPolicy::Smart, ..Default::default() };
        let expected = "می‌روم ۱۲،۳۴۵ ریال، کتاب‌ها ۱۲۳";
        assert_eq!(normalize_text(text, &options), expected);
    }

    #[test]
    fn test_user_provided_rules() {
        let text = "من توی کتاب خانه کار میکنم.";
        
        let mut slang_map = HashMap::new();
        slang_map.insert("توی".to_string(), "در".to_string());
        slang_map.insert("کار میکنم".to_string(), "کار می‌کنم".to_string());

        let options = NormalizeOptions {
            slang_map,
            zwnj_compound_words: vec!["کتاب خانه".to_string()],
            custom_rules: vec![(" من ".to_string(), " بنده ".to_string())],
            ..Default::default()
        };

        let expected = "بنده در کتاب‌خانه کار می‌کنم.";
        assert_eq!(normalize_text(text, &options), expected);
    }
}
