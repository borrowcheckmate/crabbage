/// The language of a page's `<html>` tag, from its `lang` attribute. Quotes around the value are
/// optional, and the attribute name is matched without regard to case.
pub(crate) fn html_lang(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<html")? + "<html".len();
    let end = start + lower.get(start..)?.find('>')?;
    html.get(start..end)?
        .split_whitespace()
        .find_map(|attribute| {
            let (name, value) = attribute.split_once('=')?;
            name.eq_ignore_ascii_case("lang")
                .then(|| value.trim_matches(['"', '\'']).to_owned())
        })
        .filter(|language| !language.is_empty())
}
