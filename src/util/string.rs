use lazy_static::lazy_static;
use regex::Regex;
use strsim::normalized_damerau_levenshtein;

/// Convert u64 amount of cents into $xx.xx format
pub fn currency_string(cents: u64) -> String {
    let dollars = cents / 100;
    let cents_remainder = cents % 100;

    format!("${}.{:02}", dollars, cents_remainder)
}

/// Title case conversion that capitalizes between hyphens words seperated by hyphens
pub fn to_title_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    for c in s.chars() {
        if capitalize_next {
            result.push_str(&c.to_uppercase().to_string());
            capitalize_next = false;
        } else {
            result.push(c);
        }
        if c == ' ' || c == '-' {
            capitalize_next = true;
        }
    }
    result
}

lazy_static! {
    static ref REMOVE_SKIN_NAME_REGEX: Regex =
        Regex::new(r"[^a-zA-Z0-9&ö龍王弐壱]+").expect("Failed to compile non-alphanumeric regex");
}

/// Function that removes name formatting of a CS2 item skin:
pub fn normalize_item_name(s: &str) -> String {
    let mut result = s.to_lowercase();
    result = REMOVE_SKIN_NAME_REGEX.replace_all(&result, "").to_string();
    result.trim().to_string()
}

pub fn find_closest_match(query: &String, options: &Vec<String>, threshold: f64) -> Option<String> {
    let mut highest_ratio = 0.0;
    let mut closest = None;
    for option in options {
        let ratio = normalized_damerau_levenshtein(query, option);
        if ratio > threshold && ratio > highest_ratio {
            highest_ratio = ratio;
            closest = Some(String::from(option));
        }
    }
    closest
}

#[cfg(test)]
mod tests {
    use crate::util::string::{currency_string, normalize_item_name, to_title_case};

    #[test]
    fn test_currency_string() {
        assert_eq!(currency_string(0), "$0.00");
        assert_eq!(currency_string(1), "$0.01");
        assert_eq!(currency_string(99), "$0.99");
        assert_eq!(currency_string(100), "$1.00");
        assert_eq!(currency_string(123), "$1.23");
        assert_eq!(currency_string(12345), "$123.45");
        assert_eq!(currency_string(1000000), "$10000.00");
        assert_eq!(currency_string(999999999), "$9999999.99");
        assert_eq!(currency_string(u64::MAX), "$184467440737095516.15");
    }

    #[test]
    fn test_normalize_item_name() {
        assert_eq!(normalize_item_name("AK-47 | Ice Coaled"), "ak47icecoaled");
        assert_eq!(normalize_item_name("Desert Eagle | Sunset Storm 弐"), "deserteaglesunsetstorm弐");
        assert_eq!(normalize_item_name("Desert Eagle | Sunset Storm 壱"), "deserteaglesunsetstorm壱");
        assert_eq!(normalize_item_name("AUG | Flame Jörmungandr"), "augflamejörmungandr");
        assert_eq!(normalize_item_name("M4A4 | 龍王 (Dragon King)"), "m4a4龍王dragonking");
        assert_eq!(normalize_item_name("Desert Eagle | Ramese's Reach"), "deserteagleramesesreach");
        assert_eq!(normalize_item_name("P250 | Black & Tan"), "p250black&tan");
        assert_eq!(normalize_item_name("Sawed Off | Kiss♥Love"), "sawedoffkisslove");
        assert_eq!(normalize_item_name("M249 | O.S.I.P.R."), "m249osipr");
        assert_eq!(normalize_item_name("Negev | dev_texture"), "negevdevtexture");
        assert_eq!(normalize_item_name("AWP | Man-o'-war"), "awpmanowar");
        assert_eq!(normalize_item_name("Galil AR | CAUTION!"), "galilarcaution");
    }

    #[test]
    fn test_title_case() {
        assert_eq!(to_title_case("factory new"), "Factory New");
        assert_eq!(to_title_case("mil-spec"), "Mil-Spec");
        assert_eq!(to_title_case("---bruh-moment"), "---Bruh-Moment");
    }
}
