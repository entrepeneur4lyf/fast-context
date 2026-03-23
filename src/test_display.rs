#[cfg(test)]
mod tests {
    use crate::parsers::LanguageId;
    use crate::utils::detect_language_id;

    #[test]
    fn test_display_trait() {
        let lang = LanguageId::Rust;
        assert_eq!(format!("{}", lang), "Rust");
        assert_eq!(lang.to_string(), "Rust");

        let lang2 = LanguageId::JavaScript;
        assert_eq!(format!("{}", lang2), "JavaScript");
        assert_eq!(lang2.to_string(), "JavaScript");
    }

    #[test]
    fn test_detect_language_casing() {
        let detected = detect_language_id("main.rs");
        assert!(detected.is_some());
        assert_eq!(detected.unwrap().to_string(), "Rust");

        let detected2 = detect_language_id("index.js");
        assert!(detected2.is_some());
        assert_eq!(detected2.unwrap().to_string(), "JavaScript");
    }
}
