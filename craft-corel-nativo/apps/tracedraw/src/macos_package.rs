//! The macOS package (`packaging/macos`): checks that `Info.plist` is a
//! well-formed property list and declares the document types Finder
//! needs to offer Craft Corel for `.tdraw` and `.cdr` files and to let the
//! user make it the default program for them (#1504). See
//! docs/behavior/about-and-startup.md.

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    const PLIST: &str = include_str!("../../../packaging/macos/Info.plist");
    const BUILD: &str = include_str!("../../../packaging/macos/build-dmg.sh");

    /// A property list value, as far as `Info.plist` uses them.
    #[derive(Debug, Clone, PartialEq)]
    enum Value {
        Str(String),
        Bool(bool),
        Array(Vec<Value>),
        Dict(BTreeMap<String, Value>),
    }

    impl Value {
        fn get(&self, key: &str) -> Option<&Value> {
            match self {
                Value::Dict(map) => map.get(key),
                _ => None,
            }
        }
        fn str(&self) -> Option<&str> {
            match self {
                Value::Str(s) => Some(s),
                _ => None,
            }
        }
        fn array(&self) -> &[Value] {
            match self {
                Value::Array(items) => items,
                _ => &[],
            }
        }
        fn strings(&self) -> Vec<&str> {
            self.array().iter().filter_map(Value::str).collect()
        }
    }

    /// Reads one value element; a dictionary must alternate `key` and a
    /// value, with no key twice, as the property list format requires.
    fn value(node: roxmltree::Node) -> Result<Value, String> {
        let children = || node.children().filter(|n| n.is_element());
        match node.tag_name().name() {
            "string" => Ok(Value::Str(node.text().unwrap_or("").to_string())),
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            "array" => children()
                .map(value)
                .collect::<Result<_, _>>()
                .map(Value::Array),
            "dict" => {
                let mut map = BTreeMap::new();
                let items: Vec<_> = children().collect();
                if items.len() % 2 != 0 {
                    return Err("a dict with a key and no value".into());
                }
                for pair in items.chunks(2) {
                    if pair[0].tag_name().name() != "key" {
                        return Err(format!("{:?} where a key belongs", pair[0].tag_name()));
                    }
                    let key = pair[0].text().unwrap_or("").to_string();
                    if map.insert(key.clone(), value(pair[1])?).is_some() {
                        return Err(format!("the key {key} twice"));
                    }
                }
                Ok(Value::Dict(map))
            }
            other => Err(format!("unexpected element {other}")),
        }
    }

    fn parse(text: &str) -> Result<Value, String> {
        let options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        };
        let doc =
            roxmltree::Document::parse_with_options(text, options).map_err(|e| e.to_string())?;
        let root = doc.root_element();
        if root.tag_name().name() != "plist" || root.attribute("version") != Some("1.0") {
            return Err("the root is not <plist version=\"1.0\">".into());
        }
        let mut values = root.children().filter(|n| n.is_element());
        let top = values.next().ok_or("an empty plist")?;
        if values.next().is_some() {
            return Err("more than one top value".into());
        }
        value(top)
    }

    fn plist() -> Value {
        parse(PLIST).unwrap_or_else(|e| panic!("Info.plist does not parse: {e}"))
    }

    /// The document type entry that claims a content type.
    fn document_type<'a>(plist: &'a Value, content_type: &str) -> &'a Value {
        let types = plist
            .get("CFBundleDocumentTypes")
            .expect("CFBundleDocumentTypes");
        types
            .array()
            .iter()
            .find(|t| {
                t.get("LSItemContentTypes")
                    .is_some_and(|c| c.strings().contains(&content_type))
            })
            .unwrap_or_else(|| panic!("no document type claims {content_type}"))
    }

    /// The type declaration (exported or imported) of an identifier.
    fn declaration<'a>(plist: &'a Value, key: &str, identifier: &str) -> &'a Value {
        plist
            .get(key)
            .unwrap_or_else(|| panic!("no {key}"))
            .array()
            .iter()
            .find(|d| d.get("UTTypeIdentifier").and_then(Value::str) == Some(identifier))
            .unwrap_or_else(|| panic!("{identifier} is not in {key}"))
    }

    fn extensions(declaration: &Value) -> Vec<&str> {
        declaration
            .get("UTTypeTagSpecification")
            .and_then(|tags| tags.get("public.filename-extension"))
            .map(Value::strings)
            .unwrap_or_default()
    }

    /// The checker itself refuses what Launch Services would not read.
    #[test]
    fn the_checker_refuses_broken_property_lists() {
        assert!(parse("<plist version=\"1.0\"><dict><key>a</key></dict></plist>").is_err());
        assert!(parse(
            "<plist version=\"1.0\"><dict><string>a</string><string>b</string></dict></plist>"
        )
        .is_err());
        assert!(parse(
            "<plist version=\"1.0\"><dict><key>a</key><true/><key>a</key><true/></dict></plist>"
        )
        .is_err());
        assert!(parse("<plist version=\"1.0\"><dict>").is_err());
        assert_eq!(
            parse("<plist version=\"1.0\"><dict><key>a</key><array><string>b</string></array></dict></plist>"),
            Ok(Value::Dict(BTreeMap::from([(
                "a".to_string(),
                Value::Array(vec![Value::Str("b".into())])
            )])))
        );
    }

    /// The bundle's identity, which the build script fills in.
    #[test]
    fn the_bundle_is_named_and_versioned() {
        let plist = plist();
        let get = |key: &str| plist.get(key).and_then(Value::str);
        assert_eq!(get("CFBundleIdentifier"), Some("dev.tracedraw.app"));
        assert_eq!(get("CFBundleExecutable"), Some("tracedraw"));
        assert_eq!(get("CFBundlePackageType"), Some("APPL"));
        assert_eq!(get("CFBundleVersion"), Some("__VERSION__"));
        assert!(BUILD.contains("s/__VERSION__/$VERSION/g"));
        assert!(BUILD.contains("packaging/macos/Info.plist"));
    }

    /// Finder resolves a file to a content type and keeps the default
    /// program per content type: "Change All" in Get Info and a double
    /// click go through it. A document type given by extension only is
    /// matched to no type the system knows, so `.cdr` files (which the
    /// system types as `com.apple.disk-image-cdr`, "DVD/CD-R Master
    /// Image") could be opened with Open With but Craft Corel could not
    /// become their default program, and a double click still went to
    /// the disk image mounter (#1504). Every document type therefore
    /// names its content types, and each content type of Craft Corel's own
    /// is declared with its extension.
    #[test]
    fn document_types_name_content_types_finder_knows() {
        let plist = plist();

        let tdraw = document_type(&plist, "dev.tracedraw.tdraw");
        assert_eq!(
            tdraw.get("CFBundleTypeRole").and_then(Value::str),
            Some("Editor")
        );
        assert_eq!(
            tdraw.get("LSHandlerRank").and_then(Value::str),
            Some("Owner")
        );
        let exported = declaration(&plist, "UTExportedTypeDeclarations", "dev.tracedraw.tdraw");
        assert_eq!(extensions(exported), ["tdraw"]);

        let cdr = document_type(&plist, "com.apple.disk-image-cdr");
        // Opened and saved, so an editor; never the default on its own,
        // since the same extension is also a disk image.
        assert_eq!(
            cdr.get("CFBundleTypeRole").and_then(Value::str),
            Some("Editor")
        );
        assert_eq!(
            cdr.get("LSHandlerRank").and_then(Value::str),
            Some("Alternate")
        );
        assert!(cdr
            .get("LSItemContentTypes")
            .unwrap()
            .strings()
            .contains(&"dev.tracedraw.cdr"));
        let imported = declaration(&plist, "UTImportedTypeDeclarations", "dev.tracedraw.cdr");
        assert_eq!(extensions(imported), ["cdr"]);

        for (key, identifier) in [
            ("UTExportedTypeDeclarations", "dev.tracedraw.tdraw"),
            ("UTImportedTypeDeclarations", "dev.tracedraw.cdr"),
        ] {
            let d = declaration(&plist, key, identifier);
            let conforms = d
                .get("UTTypeConformsTo")
                .map(Value::strings)
                .unwrap_or_default();
            assert!(
                conforms.contains(&"public.data"),
                "{identifier} conforms to {conforms:?}"
            );
            assert!(d.get("UTTypeDescription").and_then(Value::str).is_some());
        }

        // Every document type names a content type that is either
        // declared here or the system's own, and has a name and a role.
        let declared: Vec<&str> = ["UTExportedTypeDeclarations", "UTImportedTypeDeclarations"]
            .iter()
            .flat_map(|k| plist.get(k).map(Value::array).unwrap_or_default())
            .filter_map(|d| d.get("UTTypeIdentifier").and_then(Value::str))
            .collect();
        for t in plist.get("CFBundleDocumentTypes").unwrap().array() {
            assert!(t.get("CFBundleTypeName").and_then(Value::str).is_some());
            assert!(t.get("CFBundleTypeRole").and_then(Value::str).is_some());
            let types = t
                .get("LSItemContentTypes")
                .map(Value::strings)
                .unwrap_or_default();
            assert!(
                !types.is_empty(),
                "a document type by extension only: {t:?}"
            );
            for ty in types {
                assert!(
                    declared.contains(&ty) || ty.starts_with("com.apple."),
                    "{ty} is not declared"
                );
            }
        }
    }

    /// The same two formats as the Linux launcher and the Windows
    /// installers offer Craft Corel for, no more.
    #[test]
    fn the_same_formats_as_on_the_other_systems() {
        let desktop = include_str!("../../../packaging/linux/tracedraw.desktop");
        assert!(desktop.contains("MimeType=application/x-tdraw;application/x-cdr;"));
        let plist = plist();
        let mut all: Vec<&str> = ["UTExportedTypeDeclarations", "UTImportedTypeDeclarations"]
            .iter()
            .flat_map(|k| plist.get(k).map(Value::array).unwrap_or_default())
            .flat_map(extensions)
            .collect();
        all.sort();
        assert_eq!(all, ["cdr", "tdraw"]);
        let tdraw = declaration(&plist, "UTExportedTypeDeclarations", "dev.tracedraw.tdraw");
        let mime = tdraw
            .get("UTTypeTagSpecification")
            .and_then(|t| t.get("public.mime-type"))
            .map(Value::strings)
            .unwrap_or_default();
        assert_eq!(mime, ["application/x-tdraw"]);
    }
}
