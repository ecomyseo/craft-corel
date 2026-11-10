//! The browser build's fonts to fetch: the font folder, whose
//! `fonts/fonts.json` next to the page lists the families it holds and
//! their files, and the font catalog, whose `fonts/catalog.json` lists
//! every other family of the google/fonts collection by the paths of its
//! files in that repository (built by `web/font_catalog.py`). The page
//! fetches a family's files the first time a drawing's text needs it (see
//! `web::fetch_requested_fonts`), so fonts cost nothing until used.

use std::collections::BTreeMap;

/// Where the catalog's files are fetched from: the google/fonts
/// repository at one commit, through GitHub's file server (which lets
/// pages fetch them).
pub const CATALOG_HOST: &str = "https://raw.githubusercontent.com/google/fonts/";

/// The font catalog: the address its paths are under and each family's
/// files.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    pub base: String,
    pub families: BTreeMap<String, Vec<String>>,
}

impl Catalog {
    /// The addresses of a family's files (none for a family it lacks).
    pub fn urls(&self, family: &str) -> Vec<String> {
        self.families
            .get(family)
            .map(|paths| paths.iter().map(|p| catalog_url(&self.base, p)).collect())
            .unwrap_or_default()
    }
}

/// `catalog.json`: `base`, the repository at a commit under
/// [`CATALOG_HOST`] (a 40-digit hexadecimal commit, so files never change
/// under the page), and `families`, each with the plain paths of its font
/// files (`ofl/abel/Abel-Regular.ttf`). Anything else gives an empty
/// catalog; paths that are not plain are dropped, and families left
/// without files.
pub fn parse_catalog(bytes: &[u8]) -> Catalog {
    #[derive(serde::Deserialize)]
    struct Raw {
        base: String,
        families: BTreeMap<String, serde_json::Value>,
    }
    let Ok(raw) = serde_json::from_slice::<Raw>(bytes) else {
        return Catalog::default();
    };
    let commit = raw
        .base
        .strip_prefix(CATALOG_HOST)
        .and_then(|rest| rest.strip_suffix('/'))
        .unwrap_or("");
    if commit.len() != 40 || !commit.chars().all(|c| c.is_ascii_hexdigit()) {
        return Catalog::default();
    }
    let families = raw
        .families
        .into_iter()
        .filter_map(|(family, files)| {
            let files: Vec<String> = files
                .as_array()?
                .iter()
                .filter_map(|f| f.as_str())
                .filter(|f| plain_catalog_path(f))
                .map(String::from)
                .collect();
            (!family.trim().is_empty() && !files.is_empty()).then_some((family, files))
        })
        .collect();
    Catalog {
        base: raw.base,
        families,
    }
}

/// A plain path in the collection: `ofl`, `apache` or `ufl`, a folder of
/// lowercase letters, digits and `_`, then a font file name of letters,
/// digits, `.`, `_`, `-` and the `[`, `,` and `]` of variable fonts' axes.
fn plain_catalog_path(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    let [top, folder, file] = parts.as_slice() else {
        return false;
    };
    let lower = file.to_ascii_lowercase();
    matches!(*top, "ofl" | "apache" | "ufl")
        && !folder.is_empty()
        && folder
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !file.is_empty()
        && file.len() <= 128
        && !file.starts_with('.')
        && !file.contains("..")
        && file
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '[' | ']' | ','))
        && [".ttf", ".otf"].iter().any(|e| lower.ends_with(e))
}

/// The address of a catalog path: the base, then the path with the
/// brackets and commas of variable fonts' names escaped.
pub fn catalog_url(base: &str, path: &str) -> String {
    let mut url = String::with_capacity(base.len() + path.len() + 8);
    url.push_str(base);
    for c in path.chars() {
        match c {
            '[' => url.push_str("%5B"),
            ']' => url.push_str("%5D"),
            ',' => url.push_str("%2C"),
            c => url.push(c),
        }
    }
    url
}

/// The folder's families and their file names, from `fonts.json`: an
/// object mapping each family to a list of file names. File names that
/// could leave the folder or are not font files are dropped, and so are
/// families left without files; anything else that does not read gives
/// an empty list.
pub fn parse_list(bytes: &[u8]) -> BTreeMap<String, Vec<String>> {
    let Ok(map) = serde_json::from_slice::<BTreeMap<String, serde_json::Value>>(bytes) else {
        return BTreeMap::new();
    };
    map.into_iter()
        .filter_map(|(family, files)| {
            let files: Vec<String> = files
                .as_array()?
                .iter()
                .filter_map(|f| f.as_str())
                .filter(|f| safe_file_name(f))
                .map(String::from)
                .collect();
            (!family.trim().is_empty() && !files.is_empty()).then_some((family, files))
        })
        .collect()
}

/// A plain font file name in the folder: letters, digits, `.`, `_` and
/// `-`, ending in a font extension, no path parts.
fn safe_file_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !name.is_empty()
        && name.len() <= 128
        && !name.starts_with('.')
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        && [".ttf", ".otf", ".ttc"].iter().any(|e| lower.ends_with(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_font_list_reads_families_and_keeps_files_in_the_folder() {
        let list = parse_list(
            br#"{
                "Carlito": ["Carlito-Regular.ttf", "Carlito-Bold.ttf"],
                "Open Sans": ["OpenSans-VF.ttf", "../secret.ttf", "/etc/x.ttf",
                              "a/b.ttf", "notes.txt", ".hidden.ttf", 3],
                "Nothing Left": ["readme.md"],
                "Not a list": "Font.ttf",
                "": ["Empty-Name.ttf"]
            }"#,
        );
        assert_eq!(list.len(), 2, "{list:?}");
        assert_eq!(list["Carlito"], ["Carlito-Regular.ttf", "Carlito-Bold.ttf"]);
        assert_eq!(list["Open Sans"], ["OpenSans-VF.ttf"]);
        // Damaged lists give nothing, never a panic.
        assert!(parse_list(b"").is_empty());
        assert!(parse_list(b"[1, 2]").is_empty());
        assert!(parse_list(br#"{"A": ["A.ttf""#).is_empty());
    }

    #[test]
    fn the_catalog_reads_plain_paths_under_one_commit() {
        let base = format!("{CATALOG_HOST}bd8f81ddb5c74d5c8897b36ad88b440266245103/");
        let json = format!(
            r#"{{"base": "{base}", "families": {{
                "Abel": ["ofl/abel/Abel-Regular.ttf"],
                "Roboto": ["ofl/roboto/Roboto[wdth,wght].ttf", "ofl/roboto/Roboto-Italic[wdth,wght].ttf"],
                "Bad": ["../x.ttf", "ofl/../x.ttf", "other/abel/A.ttf", "ofl/Abel/A.ttf",
                        "ofl/abel/x/A.ttf", "ofl/abel/A.woff2", "ofl/abel/.A.ttf", 7],
                "": ["ofl/abel/Abel-Regular.ttf"]
            }}}}"#
        );
        let c = parse_catalog(json.as_bytes());
        assert_eq!(c.base, base);
        assert_eq!(c.families.len(), 2, "{c:?}");
        assert_eq!(
            c.urls("Roboto"),
            [
                format!("{base}ofl/roboto/Roboto%5Bwdth%2Cwght%5D.ttf"),
                format!("{base}ofl/roboto/Roboto-Italic%5Bwdth%2Cwght%5D.ttf"),
            ]
        );
        assert_eq!(c.urls("Abel"), [format!("{base}ofl/abel/Abel-Regular.ttf")]);
        assert!(c.urls("Nothing").is_empty());
        // Another host, a branch instead of a commit, or a damaged file:
        // no catalog.
        for bad in [
            r#"{"base": "https://example.com/google/fonts/bd8f81ddb5c74d5c8897b36ad88b440266245103/", "families": {"Abel": ["ofl/abel/Abel-Regular.ttf"]}}"#,
            r#"{"base": "https://raw.githubusercontent.com/google/fonts/main/", "families": {"Abel": ["ofl/abel/Abel-Regular.ttf"]}}"#,
            r#"{"base": 3}"#,
            "",
        ] {
            assert_eq!(parse_catalog(bad.as_bytes()), Catalog::default(), "{bad}");
        }
    }

    /// The catalog script reads a family's name and files from its
    /// METADATA.pb the way the page expects them.
    #[test]
    fn the_catalog_script_and_the_page_agree() {
        let script = include_str!("../../../web/font_catalog.py");
        assert!(script.contains(CATALOG_HOST));
        assert!(script.contains("catalog.json"));
        let ci = include_str!("../../../.github/workflows/build.yml");
        assert!(ci.contains("web/font_catalog.py web/fonts.txt site/fonts"));
    }
}
