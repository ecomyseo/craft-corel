//! Exercise every private Encoding accessor through the real DLL.
fn main() {
    use encoding_rs::*;
    let accessors: &[(&str, fn() -> &'static Encoding)] = &[
        ("IBM866", __craft_get_ibm866_init),
        ("ISO-8859-13", __craft_get_iso_8859_13_init),
        ("ISO-8859-2", __craft_get_iso_8859_2_init),
        ("ISO-8859-4", __craft_get_iso_8859_4_init),
        ("ISO-8859-5", __craft_get_iso_8859_5_init),
        ("ISO-8859-6", __craft_get_iso_8859_6_init),
        ("ISO-8859-7", __craft_get_iso_8859_7_init),
        ("ISO-8859-8", __craft_get_iso_8859_8_init),
        ("KOI8-U", __craft_get_koi8_u_init),
        ("windows-1250", __craft_get_windows_1250_init),
        ("windows-1251", __craft_get_windows_1251_init),
        ("windows-1252", __craft_get_windows_1252_init),
        ("windows-1253", __craft_get_windows_1253_init),
        ("windows-1254", __craft_get_windows_1254_init),
        ("windows-1255", __craft_get_windows_1255_init),
        ("windows-1256", __craft_get_windows_1256_init),
        ("windows-1257", __craft_get_windows_1257_init),
        ("windows-1258", __craft_get_windows_1258_init),
        ("windows-874", __craft_get_windows_874_init),
    ];
    for (label, get) in accessors {
        let expected = Encoding::for_label_no_replacement(label.as_bytes()).expect("known label");
        assert!(std::ptr::eq(get(), expected), "same Encoding instance: {label}");
        let (encoded, _, failed) = get().encode("Craft Plumilla 123");
        assert!(!failed);
        let (decoded, _, failed) = get().decode(&encoded);
        assert!(!failed);
        assert_eq!(decoded, "Craft Plumilla 123");
        let mut detector = chardetng::EncodingDetector::new();
        detector.feed(&encoded, true);
        assert_eq!(detector.guess(None, true).name(), "UTF-8");
        let (sample, tld): (&str, &[u8]) = match *label {
            "IBM866" | "ISO-8859-5" | "windows-1251" => (
                "Русский язык сохраняет исходный текст документа, буквы и знаки препинания. ", b"ru"),
            "KOI8-U" => ("Українська мова зберігає всі літери та інформацію документа. ", b"ua"),
            "ISO-8859-2" | "windows-1250" => ("Příliš žluťoučký kůň úpěl ďábelské ódy. Český text obsahuje všechna písmena. ", b"cz"),
            "ISO-8859-4" | "ISO-8859-13" | "windows-1257" => ("Lietuvių kalba išsaugo informaciją apie gražų miestą ir žmonių gyvenimą. ", b"lt"),
            "ISO-8859-6" | "windows-1256" => ("اللغة العربية لغة جميلة وهذا النص يحتفظ بالمعلومات والكلمات في المستند. ", b"eg"),
            "ISO-8859-7" | "windows-1253" => ("Η ελληνική γλώσσα διατηρεί τις πληροφορίες και τα γράμματα του εγγράφου. ", b"gr"),
            "ISO-8859-8" | "windows-1255" => ("השפה העברית שומרת על המידע ועל האותיות והסימנים במסמך הזה. ", b"il"),
            "windows-1252" => ("El diseño español conserva la información, las tildes y la ñ. ", b"es"),
            "windows-1254" => ("Türkçe metin doğru bilgiyi, ışığı ve güzel şehrin bütün harflerini korur. ", b"tr"),
            "windows-1258" => ("Tiê\u{301}ng Viê\u{323}t và thông tin trong tài liê\u{323}u. ", b"vn"),
            "windows-874" => ("ภาษาไทยเป็นภาษาที่สวยงามและข้อความนี้รักษาข้อมูลของเอกสารไว้ทั้งหมด ", b"th"),
            _ => unreachable!("all labels covered"),
        };
        let sample = sample.repeat(20);
        let (bytes, _, failed) = get().encode(&sample);
        assert!(!failed, "sample representable in {label}");
        let (decoded, _, failed) = get().decode(&bytes);
        assert!(!failed);
        assert_eq!(decoded, sample, "non-ASCII decoding: {label}");
        let mut detector = chardetng::EncodingDetector::new();
        detector.feed(&bytes, true);
        let guessed = detector.guess(Some(tld), true);
        let (detected_text, _, failed) = guessed.decode(&bytes);
        assert!(!failed, "detected {label} as {}", guessed.name());
        assert_eq!(detected_text, sample, "detect {label} as {}", guessed.name());
        println!("non-ASCII input {label}, detected {}: exact text preserved", guessed.name());
    }
    let sample = "El diseño español conserva la información, las tildes y la ñ. ".repeat(20);
    let (bytes, _, failed) = WINDOWS_1252.encode(&sample);
    assert!(!failed);
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&bytes, true);
    assert_eq!(detector.guess(Some(b"es"), true).name(), "windows-1252");
    println!("19 accessors preserve identity, ASCII and non-ASCII round trips and detection; Spanish Windows-1252 detection passed");
}
