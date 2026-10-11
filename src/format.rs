// Mise en forme des nombres et des dates à la française pour l'interface
use chrono::{Datelike, NaiveDateTime, Timelike};

/// Nombre avec virgule décimale, deux décimales au plus et espace fine insécable entre les milliers.
fn format_number(value: f64) -> String {
    let rounded = format!("{:.2}", value.abs());
    let (integer, decimals) = rounded.split_once('.').unwrap_or((&rounded, ""));
    let decimals = decimals.trim_end_matches('0');

    let mut result = String::new();
    // « -0,00 » n'a pas de sens : le signe ne s'affiche que si l'arrondi n'est pas nul
    if value < 0.0 && rounded.chars().any(|c| c.is_ascii_digit() && c != '0') {
        result.push('-');
    }
    for (index, digit) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index) % 3 == 0 {
            result.push('\u{202f}');
        }
        result.push(digit);
    }
    if !decimals.is_empty() {
        result.push(',');
        result.push_str(decimals);
    }
    result
}

/// Taille de fichier lisible, par exemple « 1,5 Ko ».
pub fn format_file_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 octet".to_owned();
    }
    const UNITS: [&str; 4] = ["octets", "Ko", "Mo", "Go"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{} {}", format_number(value), UNITS[unit])
}

/// Gain de compression, par exemple « 91,38 % ».
pub fn format_percent(ratio: f64) -> String {
    format!("{} %", format_number(ratio))
}

/// Dimensions en pixels, par exemple « 4000 × 3000 ».
pub fn format_dimensions((width, height): (u32, u32)) -> String {
    format!("{width} × {height}")
}

/// Date et heure courtes, par exemple « 11 oct. 2026, 18:29 ».
pub fn format_date(date: NaiveDateTime) -> String {
    const MONTHS: [&str; 12] = [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ];
    format!(
        "{} {} {}, {:02}:{:02}",
        date.day(),
        MONTHS[date.month0() as usize],
        date.year(),
        date.hour(),
        date.minute()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_dates_in_french() {
        let date = |month, day, hour, minute| {
            chrono::NaiveDate::from_ymd_opt(2026, month, day)
                .unwrap()
                .and_hms_opt(hour, minute, 0)
                .unwrap()
        };

        assert_eq!(format_date(date(10, 11, 18, 29)), "11 oct. 2026, 18:29");
        assert_eq!(format_date(date(3, 1, 9, 5)), "1 mars 2026, 09:05");
    }

    #[test]
    fn formats_sizes_with_french_units() {
        assert_eq!(format_file_size(0), "0 octet");
        assert_eq!(format_file_size(512), "512 octets");
        assert_eq!(format_file_size(1536), "1,5 Ko");
        assert_eq!(format_file_size(5 * 1024 * 1024), "5 Mo");
    }

    #[test]
    fn caps_at_the_largest_unit() {
        assert_eq!(format_file_size(2 * 1024u64.pow(4)), "2\u{202f}048 Go");
    }

    #[test]
    fn formats_percent_in_french() {
        assert_eq!(format_percent(91.38), "91,38 %");
        assert_eq!(format_percent(-12.5), "-12,5 %");
        assert_eq!(format_percent(-0.001), "0 %");
    }

    #[test]
    fn formats_dimensions() {
        assert_eq!(format_dimensions((4000, 3000)), "4000 × 3000");
    }
}
