//! The PROJECT stream (MS-OVBA 2.3.1): `key=value` lines naming the
//! modules by kind (`Module=`, `Class=`, `BaseClass=` for forms,
//! `Document=`), the help file, and the protection state, kept in the
//! lightly scrambled `CMG`, `DPB` and `GC` values (MS-OVBA 2.4.3).

/// How the project is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Protection {
    /// The project has a password.
    pub password: bool,
    /// The project is locked for viewing (its modules are hidden in the
    /// editor that saved it).
    pub locked_for_viewing: bool,
    /// The protection state says the user protected it.
    pub user_protected: bool,
}

impl Protection {
    /// Any kind of protection. The source of a protected project is still
    /// stored in the clear; the protection only asks editors to hide it.
    pub fn any(&self) -> bool {
        self.password || self.locked_for_viewing || self.user_protected
    }
}

/// What the PROJECT stream says.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub help_file: String,
    /// Module names by kind.
    pub standard: Vec<String>,
    pub classes: Vec<String>,
    pub forms: Vec<String>,
    pub documents: Vec<String>,
    pub protection: Protection,
}

/// The value of a `key="value"` line without its quotes.
fn unquote(v: &str) -> String {
    let v = v.trim();
    v.strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(v)
        .to_string()
}

/// Hex text as bytes (`None` when it is not hex).
fn hex(text: &str) -> Option<Vec<u8>> {
    let t = text.trim();
    if !t.len().is_multiple_of(2) {
        return None;
    }
    (0..t.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(t.get(i..i + 2)?, 16).ok())
        .collect()
}

/// Undo the scrambling of a `CMG`, `DPB` or `GC` value (MS-OVBA
/// 2.4.3.3). `None` when the value is too short or its version is not 2.
pub fn decrypt(text: &str) -> Option<Vec<u8>> {
    let data = hex(text)?;
    if data.len() < 3 {
        return None;
    }
    let seed = data[0];
    let version_enc = data[1];
    let key_enc = data[2];
    if seed ^ version_enc != 2 {
        return None;
    }
    let mut unencrypted1 = seed ^ key_enc;
    let mut encrypted1 = key_enc;
    let mut encrypted2 = version_enc;
    let mut pos = 3;
    let mut next = |pos: &mut usize| -> Option<u8> {
        let enc = *data.get(*pos)?;
        *pos += 1;
        let byte = enc ^ encrypted2.wrapping_add(unencrypted1);
        encrypted2 = encrypted1;
        encrypted1 = enc;
        unencrypted1 = byte;
        Some(byte)
    };
    for _ in 0..(seed & 6) / 2 {
        next(&mut pos)?;
    }
    let mut length = 0usize;
    for i in 0..4 {
        length |= (next(&mut pos)? as usize) << (8 * i);
    }
    if length > 4096 {
        return None;
    }
    let mut out = Vec::with_capacity(length);
    for _ in 0..length {
        out.push(next(&mut pos)?);
    }
    Some(out)
}

/// Scramble `data` as a `CMG`, `DPB` or `GC` value for project `id`
/// (MS-OVBA 2.4.3.2), with `seed` as the random byte.
pub fn encrypt(data: &[u8], id: &str, seed: u8) -> String {
    let version = 2u8;
    let key = id.bytes().fold(0u8, |k, b| k.wrapping_add(b));
    let version_enc = seed ^ version;
    let key_enc = seed ^ key;
    let mut out = vec![seed, version_enc, key_enc];
    let mut unencrypted1 = key;
    let mut encrypted1 = key_enc;
    let mut encrypted2 = version_enc;
    let mut push = |byte: u8, out: &mut Vec<u8>| {
        let enc = byte ^ encrypted2.wrapping_add(unencrypted1);
        out.push(enc);
        encrypted2 = encrypted1;
        encrypted1 = enc;
        unencrypted1 = byte;
    };
    for _ in 0..(seed & 6) / 2 {
        push(7, &mut out);
    }
    for i in 0..4 {
        push((data.len() >> (8 * i)) as u8, &mut out);
    }
    for &b in data {
        push(b, &mut out);
    }
    out.iter().map(|b| format!("{b:02X}")).collect()
}

/// Read the PROJECT stream's text.
pub fn parse(text: &str) -> ProjectInfo {
    let mut info = ProjectInfo::default();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            // [Host Extender Info] and [Workspace] follow the properties.
            break;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim().to_ascii_lowercase().as_str() {
            "id" => info.id = unquote(value),
            "name" => info.name = unquote(value),
            "description" => info.description = unquote(value),
            "helpfile" => info.help_file = unquote(value),
            "module" => info.standard.push(value.trim().to_string()),
            "class" => info.classes.push(value.trim().to_string()),
            "baseclass" => info.forms.push(value.trim().to_string()),
            "document" => {
                let name = value.split('/').next().unwrap_or("").trim();
                info.documents.push(name.to_string());
            }
            "cmg" => {
                if let Some(state) = decrypt(&unquote(value)) {
                    let bits = state
                        .get(..4)
                        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                        .unwrap_or(0);
                    info.protection.user_protected = bits & 1 != 0;
                }
            }
            "dpb" => {
                if let Some(pw) = decrypt(&unquote(value)) {
                    // One zero byte: no password.
                    info.protection.password = !(pw.len() == 1 && pw[0] == 0) && !pw.is_empty();
                }
            }
            "gc" => {
                if let Some(v) = decrypt(&unquote(value)) {
                    info.protection.locked_for_viewing = v.first() == Some(&0);
                }
            }
            _ => {}
        }
    }
    info
}

/// The PROJECT stream of a project, for writing test projects and
/// projects back: the modules by kind and, when `protection` asks for
/// it, a password and the locked state.
pub fn write(info: &ProjectInfo) -> String {
    let mut s = String::new();
    let id = if info.id.is_empty() {
        "{00000000-0000-0000-0000-000000000000}"
    } else {
        &info.id
    };
    s += &format!("ID=\"{id}\"\r\n");
    for d in &info.documents {
        s += &format!("Document={d}/&H00000000\r\n");
    }
    for m in &info.standard {
        s += &format!("Module={m}\r\n");
    }
    for c in &info.classes {
        s += &format!("Class={c}\r\n");
    }
    if !info.forms.is_empty() {
        s += "Package={AC9F2F90-E877-11CE-9F68-00AA00574A4F}\r\n";
    }
    for f in &info.forms {
        s += &format!("BaseClass={f}\r\n");
    }
    s += &format!("Name=\"{}\"\r\n", info.name);
    s += "HelpContextID=\"0\"\r\nVersionCompatible32=\"393222000\"\r\n";
    let p = info.protection;
    let state: u32 = if p.user_protected { 1 } else { 0 };
    s += &format!("CMG=\"{}\"\r\n", encrypt(&state.to_le_bytes(), id, 0x31));
    let password: Vec<u8> = if p.password {
        // A password hash record: reserved 0xFF, then key and hash bytes
        // (the hash itself is not checked here), ending with zero.
        let mut v = vec![0xFF, 0x20, 0x00, 0x00];
        v.extend_from_slice(&[0x11; 4]);
        v.extend_from_slice(&[0x22; 20]);
        v.push(0);
        v
    } else {
        vec![0]
    };
    s += &format!("DPB=\"{}\"\r\n", encrypt(&password, id, 0x52));
    let visible = if p.locked_for_viewing { 0x00 } else { 0xFF };
    s += &format!("GC=\"{}\"\r\n", encrypt(&[visible], id, 0x64));
    s += "\r\n[Host Extender Info]\r\n&H00000001={3832D640-CF90-11CF-8E43-00A0C911005A};VBE;&H00000000\r\n";
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrambled_values_come_back() {
        for seed in [0u8, 1, 2, 5, 6, 7, 0x9A, 0xFF] {
            for data in [&[0u8][..], &[0xFF], &[1, 0, 0, 0], &[9; 29]] {
                let text = encrypt(data, "{12345678-1234-1234-1234-123456789ABC}", seed);
                assert_eq!(decrypt(&text).as_deref(), Some(data), "seed {seed}");
            }
        }
        assert_eq!(decrypt("zz"), None);
        assert_eq!(decrypt("0102"), None);
        assert_eq!(decrypt(""), None);
        // Version 2 but the length runs past the end.
        assert_eq!(decrypt("000200"), None);
    }

    #[test]
    fn modules_by_kind_and_protection() {
        let mut info = ProjectInfo {
            id: "{AAAAAAAA-0000-0000-0000-000000000001}".into(),
            name: "Tools".into(),
            standard: vec!["Module1".into()],
            classes: vec!["Class1".into()],
            forms: vec!["UserForm1".into()],
            documents: vec!["ThisMacroStorage".into()],
            ..Default::default()
        };
        let open = parse(&write(&info));
        assert_eq!(open.name, "Tools");
        assert_eq!(open.standard, ["Module1"]);
        assert_eq!(open.classes, ["Class1"]);
        assert_eq!(open.forms, ["UserForm1"]);
        assert_eq!(open.documents, ["ThisMacroStorage"]);
        assert!(!open.protection.any());
        info.protection = Protection {
            password: true,
            locked_for_viewing: true,
            user_protected: true,
        };
        assert_eq!(parse(&write(&info)).protection, info.protection);
    }
}
