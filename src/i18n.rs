use std::sync::LazyLock;

use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::{DefaultLocalizer, LanguageLoader, Localizer};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    loader
        .load_fallback_language(&Localizations)
        .expect("the fallback language is embedded in the binary");
    loader
});

pub fn init() {
    let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations);
    let requested = i18n_embed::DesktopLanguageRequester::requested_languages();

    match localizer.select(&requested) {
        Ok(selected) => tracing::debug!(?selected, ?requested, "loaded translations"),
        Err(error) => tracing::warn!(%error, "keeping English, the locale did not load"),
    }
}

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        ::i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        ::i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args),*)
    }};
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    const LANGUAGES: [(&str, &str); 11] = [
        ("cs", include_str!("../i18n/cs/trash.ftl")),
        ("de", include_str!("../i18n/de/trash.ftl")),
        ("es", include_str!("../i18n/es/trash.ftl")),
        ("fr", include_str!("../i18n/fr/trash.ftl")),
        ("it", include_str!("../i18n/it/trash.ftl")),
        ("nl", include_str!("../i18n/nl/trash.ftl")),
        ("pl", include_str!("../i18n/pl/trash.ftl")),
        ("pt-BR", include_str!("../i18n/pt-BR/trash.ftl")),
        ("ru", include_str!("../i18n/ru/trash.ftl")),
        ("uk", include_str!("../i18n/uk/trash.ftl")),
        ("zh-CN", include_str!("../i18n/zh-CN/trash.ftl")),
    ];

    const ENGLISH: &str = include_str!("../i18n/en/trash.ftl");

    fn message_ids(catalogue: &str) -> BTreeSet<&str> {
        catalogue
            .lines()
            .filter(|line| !line.starts_with([' ', '\t', '#', '-', '*', '[', ']']))
            .filter_map(|line| line.split_once('='))
            .map(|(id, _)| id.trim())
            .filter(|id| !id.is_empty())
            .collect()
    }

    fn message(catalogue: &str, wanted: &str) -> String {
        for line in catalogue.lines() {
            let Some((id, value)) = line.split_once('=') else {
                continue;
            };
            if id.trim() == wanted {
                return value.trim().to_string();
            }
        }
        panic!("{wanted} is missing from a catalogue");
    }

    #[test]
    fn every_language_translates_exactly_the_same_messages() {
        let english = message_ids(ENGLISH);

        for (language, catalogue) in LANGUAGES {
            let translated = message_ids(catalogue);

            assert!(
                translated == english,
                "{language} is out of step with en: missing {:?}, unknown {:?}",
                english.difference(&translated).collect::<Vec<_>>(),
                translated.difference(&english).collect::<Vec<_>>(),
            );
        }
    }

    #[test]
    fn every_catalogue_parses_and_renders() {
        use i18n_embed::LanguageLoader as _;
        use i18n_embed::unic_langid::LanguageIdentifier;

        for (language, _) in LANGUAGES {
            let id: LanguageIdentifier = language.parse().expect("a well-formed language tag");
            let loader = super::fluent_language_loader!();
            loader
                .load_languages(&super::Localizations, &[id])
                .unwrap_or_else(|error| panic!("{language} failed to load: {error}"));

            let rendered = i18n_embed_fl::fl!(loader, "empty-trash");
            assert!(
                !rendered.is_empty() && !rendered.contains('{'),
                "{language} left a placeholder unresolved: {rendered}"
            );
        }
    }

    #[test]
    fn no_menu_label_outgrows_the_popup_it_has_to_fit_in() {
        const BUDGET: usize = 28;

        for (language, catalogue) in LANGUAGES
            .into_iter()
            .chain(std::iter::once(("en", ENGLISH)))
        {
            for id in ["open-trash", "empty-trash"] {
                let label = message(catalogue, id);
                assert!(
                    label.chars().count() <= BUDGET,
                    "{language} {id} is {} characters, more than the {BUDGET} the menu row fits",
                    label.chars().count(),
                );
            }
        }
    }

    #[test]
    fn attributes_and_comments_are_not_mistaken_for_messages() {
        let ids = message_ids("# a comment = not a message\nreal-id = value\n    .attr = nope\n");

        assert!(ids.contains("real-id"), "a plain message id must be found");
        assert_eq!(ids.len(), 1, "only the plain message id counts: {ids:?}");
    }
}
