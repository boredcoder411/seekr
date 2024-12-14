pub fn get_locale() -> String {
    let language = std::env::var("LANG").unwrap_or("en".to_string());

    for locale in rust_i18n::available_locales!() {
        if language.starts_with(locale) {
            return locale.to_string();
        }
    }

    "en".to_string()
}
