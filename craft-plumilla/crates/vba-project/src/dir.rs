//! The `dir` stream of a VBA project (MS-OVBA 2.3.4.2), once expanded:
//! records of a 2-byte id, a 4-byte size and that many bytes, which give
//! the project's name, code page, references and modules.

use crate::codepage::{decode, utf16};
use crate::Error;

/// What the `dir` stream says about one module.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirModule {
    pub name: String,
    /// The module's stream in the `VBA` storage.
    pub stream_name: String,
    /// Where the compressed source starts in that stream (the p-code is
    /// before it).
    pub text_offset: u32,
    /// A procedural module (`true`), or a document, class or form one.
    pub procedural: bool,
    pub read_only: bool,
    pub private: bool,
    pub doc_string: String,
}

/// What the `dir` stream says about a reference to another library or
/// project.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DirReference {
    pub name: String,
    /// The library's identifier ("*\G{...}#2.0#0#...#Name"), when given.
    pub libid: String,
}

/// The project as the `dir` stream describes it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dir {
    pub name: String,
    pub doc_string: String,
    pub help_file: String,
    pub code_page: u16,
    pub sys_kind: u32,
    pub constants: String,
    pub references: Vec<DirReference>,
    pub modules: Vec<DirModule>,
}

fn u16_at(b: &[u8]) -> u16 {
    if b.len() >= 2 {
        u16::from_le_bytes([b[0], b[1]])
    } else {
        0
    }
}

fn u32_at(b: &[u8]) -> u32 {
    if b.len() >= 4 {
        u32::from_le_bytes([b[0], b[1], b[2], b[3]])
    } else {
        0
    }
}

/// The libid in a REFERENCEREGISTERED, REFERENCEPROJECT or
/// REFERENCEORIGINAL record's data: a 4-byte size, then that many bytes.
fn libid_of(data: &[u8], cp: u16) -> String {
    let n = u32_at(data) as usize;
    data.get(4..4 + n)
        .map(|b| decode(b, cp))
        .unwrap_or_default()
}

/// Read an expanded `dir` stream.
pub fn parse(data: &[u8]) -> Result<Dir, Error> {
    let corrupt = |what: String| Error::Corrupt(format!("dir stream: {what}"));
    let mut dir = Dir {
        code_page: 1252,
        ..Default::default()
    };
    let mut pos = 0;
    let mut module: Option<DirModule> = None;
    // A Unicode stream name, when present, is kept over the code page one.
    let mut module_unicode_stream = false;
    // A name right after a control reference's record is its extended
    // name, not a new reference.
    let mut previous = 0u16;
    let mut reference: Option<DirReference> = None;
    let mut terminated = false;
    while pos + 6 <= data.len() {
        let id = u16_at(&data[pos..]);
        let mut size = u32_at(&data[pos + 2..]) as usize;
        // PROJECTVERSION gives 4 as its size but holds 6 bytes.
        if id == 0x0009 {
            size = 6;
        }
        let start = pos + 6;
        let Some(body) = data.get(start..start + size) else {
            return Err(corrupt(format!("record 0x{id:04X} is cut short")));
        };
        pos = start + size;
        let cp = dir.code_page;
        match id {
            0x0001 => dir.sys_kind = u32_at(body),
            0x0003 => dir.code_page = u16_at(body),
            0x0004 => dir.name = decode(body, cp),
            0x0005 => dir.doc_string = decode(body, cp),
            0x0040 => dir.doc_string = utf16(body),
            0x0006 => dir.help_file = decode(body, cp),
            0x003D => {
                if !body.is_empty() {
                    dir.help_file = decode(body, cp);
                }
            }
            0x000C => dir.constants = decode(body, cp),
            0x003C => dir.constants = utf16(body),
            // A reference's name starts it.
            0x0016 if previous == 0x002F => {
                if let Some(r) = reference.as_mut() {
                    r.name = decode(body, cp);
                }
            }
            0x0016 => {
                if module.is_none() {
                    if let Some(r) = reference.take() {
                        dir.references.push(r);
                    }
                    reference = Some(DirReference {
                        name: decode(body, cp),
                        libid: String::new(),
                    });
                }
            }
            0x003E => {
                if let Some(r) = reference.as_mut() {
                    r.name = utf16(body);
                }
            }
            0x000D | 0x000E | 0x0033 => {
                if let Some(r) = reference.as_mut() {
                    if r.libid.is_empty() {
                        r.libid = libid_of(body, cp);
                    }
                }
            }
            0x002F => {
                // REFERENCECONTROL: the libid follows a 4-byte size.
                if let Some(r) = reference.as_mut() {
                    if r.libid.is_empty() {
                        r.libid = libid_of(body, cp);
                    }
                }
            }
            0x000F => {
                if let Some(r) = reference.take() {
                    dir.references.push(r);
                }
            }
            0x0019 => {
                if let Some(r) = reference.take() {
                    dir.references.push(r);
                }
                if let Some(m) = module.take() {
                    dir.modules.push(m);
                }
                module = Some(DirModule {
                    name: decode(body, cp),
                    procedural: true,
                    ..Default::default()
                });
                module_unicode_stream = false;
            }
            0x0047 => {
                if let Some(m) = module.as_mut() {
                    if !body.is_empty() {
                        m.name = utf16(body);
                    }
                }
            }
            0x001A => {
                if let Some(m) = module.as_mut() {
                    if !module_unicode_stream {
                        m.stream_name = decode(body, cp);
                    }
                }
            }
            0x0032 => {
                if let Some(m) = module.as_mut() {
                    if !body.is_empty() {
                        m.stream_name = utf16(body);
                        module_unicode_stream = true;
                    }
                }
            }
            0x001C => {
                if let Some(m) = module.as_mut() {
                    m.doc_string = decode(body, cp);
                }
            }
            0x0048 => {
                if let Some(m) = module.as_mut() {
                    m.doc_string = utf16(body);
                }
            }
            0x0031 => {
                if let Some(m) = module.as_mut() {
                    m.text_offset = u32_at(body);
                }
            }
            0x0021 => {
                if let Some(m) = module.as_mut() {
                    m.procedural = true;
                }
            }
            0x0022 => {
                if let Some(m) = module.as_mut() {
                    m.procedural = false;
                }
            }
            0x0025 => {
                if let Some(m) = module.as_mut() {
                    m.read_only = true;
                }
            }
            0x0028 => {
                if let Some(m) = module.as_mut() {
                    m.private = true;
                }
            }
            0x002B => {
                if let Some(m) = module.take() {
                    dir.modules.push(m);
                }
            }
            0x0010 => {
                terminated = true;
                break;
            }
            _ => {}
        }
        previous = id;
    }
    if let Some(r) = reference.take() {
        dir.references.push(r);
    }
    if let Some(m) = module.take() {
        dir.modules.push(m);
    }
    if !terminated && dir.modules.is_empty() && dir.name.is_empty() {
        return Err(corrupt("no project information".into()));
    }
    Ok(dir)
}

/// Records for a `dir` stream (before compression), for building test
/// projects and for writing projects back.
pub struct DirWriter {
    out: Vec<u8>,
    cp: u16,
}

impl DirWriter {
    /// The project information records of a 32-bit Windows project.
    pub fn new(name: &str, cp: u16) -> DirWriter {
        let mut w = DirWriter {
            out: Vec::new(),
            cp,
        };
        w.record(0x0001, &1u32.to_le_bytes());
        w.record(0x0002, &0x409u32.to_le_bytes());
        w.record(0x0014, &0x409u32.to_le_bytes());
        w.record(0x0003, &cp.to_le_bytes());
        let n = crate::codepage::encode(name, cp);
        w.record(0x0004, &n);
        w.record(0x0005, &[]);
        w.record(0x0040, &[]);
        w.record(0x0006, &[]);
        w.record(0x003D, &[]);
        w.record(0x0007, &0u32.to_le_bytes());
        w.record(0x0008, &0u32.to_le_bytes());
        // PROJECTVERSION: size 4, then 6 bytes.
        w.out.extend_from_slice(&0x0009u16.to_le_bytes());
        w.out.extend_from_slice(&4u32.to_le_bytes());
        w.out.extend_from_slice(&1u32.to_le_bytes());
        w.out.extend_from_slice(&0u16.to_le_bytes());
        w.record(0x000C, &[]);
        w.record(0x003C, &[]);
        w
    }

    fn record(&mut self, id: u16, data: &[u8]) {
        self.out.extend_from_slice(&id.to_le_bytes());
        self.out
            .extend_from_slice(&(data.len() as u32).to_le_bytes());
        self.out.extend_from_slice(data);
    }

    fn unicode(text: &str) -> Vec<u8> {
        text.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
    }

    /// A registered type library reference.
    pub fn reference(&mut self, name: &str, libid: &str) {
        let cp = self.cp;
        self.record(0x0016, &crate::codepage::encode(name, cp));
        self.record(0x003E, &Self::unicode(name));
        let lib = crate::codepage::encode(libid, cp);
        let mut body = (lib.len() as u32).to_le_bytes().to_vec();
        body.extend_from_slice(&lib);
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&0u16.to_le_bytes());
        self.record(0x000D, &body);
    }

    /// Start the module list.
    pub fn modules(&mut self, count: u16) {
        self.record(0x000F, &count.to_le_bytes());
        self.record(0x0013, &0xFFFFu16.to_le_bytes());
    }

    /// One module's records.
    pub fn module(&mut self, name: &str, stream: &str, offset: u32, procedural: bool) {
        let cp = self.cp;
        self.record(0x0019, &crate::codepage::encode(name, cp));
        self.record(0x0047, &Self::unicode(name));
        self.record(0x001A, &crate::codepage::encode(stream, cp));
        self.record(0x0032, &Self::unicode(stream));
        self.record(0x001C, &[]);
        self.record(0x0048, &[]);
        self.record(0x0031, &offset.to_le_bytes());
        self.record(0x001E, &0u32.to_le_bytes());
        self.record(0x002C, &0xFFFFu16.to_le_bytes());
        self.record(if procedural { 0x0021 } else { 0x0022 }, &[]);
        self.record(0x002B, &[]);
    }

    /// The finished stream (uncompressed).
    pub fn finish(mut self) -> Vec<u8> {
        self.record(0x0010, &[]);
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modules_references_and_the_code_page_are_read() {
        let mut w = DirWriter::new("Tools", 1251);
        w.reference(
            "stdole",
            "*\\G{00020430-0000-0000-C000-000000000046}#2.0#0#stdole2.tlb#OLE",
        );
        w.modules(2);
        w.module("Модуль1", "Модуль1", 1234, true);
        w.module("UserForm1", "UserForm1", 99, false);
        let d = parse(&w.finish()).unwrap();
        assert_eq!(d.name, "Tools");
        assert_eq!(d.code_page, 1251);
        assert_eq!(d.references.len(), 1);
        assert_eq!(d.references[0].name, "stdole");
        assert!(d.references[0].libid.starts_with("*\\G{00020430"));
        assert_eq!(d.modules.len(), 2);
        assert_eq!(d.modules[0].name, "Модуль1");
        assert_eq!(d.modules[0].stream_name, "Модуль1");
        assert_eq!(d.modules[0].text_offset, 1234);
        assert!(d.modules[0].procedural);
        assert!(!d.modules[1].procedural);
    }

    #[test]
    fn a_record_running_past_the_end_is_an_error() {
        let mut data = DirWriter::new("P", 1252).finish();
        data.truncate(data.len() - 6);
        data.extend_from_slice(&[0x04, 0x00, 0xFF, 0x00, 0x00, 0x00, b'x']);
        assert!(parse(&data).is_err());
        assert!(parse(&[]).is_err());
    }
}
