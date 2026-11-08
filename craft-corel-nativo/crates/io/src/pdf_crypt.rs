//! PDF password protection: the standard security handler of ISO 32000-2
//! (section 7.6.4), revision 6, with AES-256 for strings and streams.
//!
//! A protected file has a file key of 32 random bytes. The user password
//! (to open the file; may be empty) and the owner password (to change the
//! permissions) each unlock a wrapped copy of that key (/UE, /OE) after a
//! salted, iterated hash checks them (/U, /O). /Perms holds the
//! permissions encrypted with the file key so they cannot be edited
//! unnoticed. Every string and stream of the file is then encrypted with
//! AES-256 in CBC mode, a random 16-byte IV in front, PKCS #7 padding.
//!
//! Reading: [`Decryptor`] opens files of every revision of that handler
//! (2 to 6: RC4 with 40 to 128-bit keys, AES-128 and AES-256) with a
//! password, usually the empty one of a file that opens without asking.

use aes::cipher::{
    block_padding::NoPadding, block_padding::Pkcs7, BlockDecryptMut, BlockEncrypt, BlockEncryptMut,
    KeyInit, KeyIvInit,
};
use lopdf::{Dictionary, Object, ObjectId};
use md5::Md5;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// A source of random bytes: fills the buffer it is given.
pub type Random = Box<dyn FnMut(&mut [u8])>;

/// What a reader may do with a protected file without the owner password.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    pub print: bool,
    /// Print at full quality (otherwise a low-resolution print only).
    pub print_high_quality: bool,
    pub copy: bool,
    pub modify: bool,
    pub annotate: bool,
    pub fill_forms: bool,
    pub assemble: bool,
    /// Text extraction for accessibility tools.
    pub accessibility: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Permissions {
            print: true,
            print_high_quality: true,
            copy: true,
            modify: true,
            annotate: true,
            fill_forms: true,
            assemble: true,
            accessibility: true,
        }
    }
}

impl Permissions {
    /// The /P value: bits 3 to 6 and 9 to 12 for the permissions, bits 7
    /// and 8 and 13 to 32 set as the standard requires, bits 1 and 2 clear.
    pub fn p_value(&self) -> i32 {
        let mut p: u32 = 0xFFFF_F0C0;
        let mut set = |on: bool, bit: u32| {
            if on {
                p |= 1 << (bit - 1);
            }
        };
        set(self.print, 3);
        set(self.modify, 4);
        set(self.copy, 5);
        set(self.annotate, 6);
        set(self.fill_forms, 9);
        set(self.accessibility, 10);
        set(self.assemble, 11);
        set(self.print && self.print_high_quality, 12);
        p as i32
    }
}

/// The passwords and permissions of a protected file.
#[derive(Clone, PartialEq, Eq)]
pub struct Security {
    /// Needed to open the file; empty opens it without asking.
    pub user_password: String,
    /// Needed to change the permissions; empty means the user password.
    pub owner_password: String,
    pub permissions: Permissions,
}

impl std::fmt::Debug for Security {
    // The passwords stay out of any log.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Security")
            .field("user_password", &(!self.user_password.is_empty()))
            .field("owner_password", &(!self.owner_password.is_empty()))
            .field("permissions", &self.permissions)
            .finish()
    }
}

/// Passwords are UTF-8, at most 127 bytes (cut at a character boundary).
fn prepare(password: &str) -> Vec<u8> {
    let mut end = password.len().min(127);
    while !password.is_char_boundary(end) {
        end -= 1;
    }
    password.as_bytes()[..end].to_vec()
}

fn aes128_cbc_nopad(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    let mut buf = data.to_vec();
    let len = buf.len();
    match cbc::Encryptor::<aes::Aes128>::new_from_slices(key, iv) {
        Ok(enc) => match enc.encrypt_padded_mut::<NoPadding>(&mut buf, len) {
            Ok(out) => out.to_vec(),
            Err(_) => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}

fn aes256_cbc_nopad(key: &[u8], iv: &[u8], data: &[u8]) -> Vec<u8> {
    let mut buf = data.to_vec();
    let len = buf.len();
    match cbc::Encryptor::<aes::Aes256>::new_from_slices(key, iv) {
        Ok(enc) => match enc.encrypt_padded_mut::<NoPadding>(&mut buf, len) {
            Ok(out) => out.to_vec(),
            Err(_) => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}

/// The password hash of revision 6 (ISO 32000-2, algorithm 2.B): SHA-256
/// of password, salt and `udata` (the 48-byte /U for owner values), then
/// at least 64 rounds that encrypt 64 copies of password, key and `udata`
/// with AES-128 and hash the result with SHA-256, -384 or -512 (chosen by
/// the first 16 bytes modulo 3), until the last byte of the round's
/// output is no greater than the round number minus 32.
pub fn hash_r6(password: &[u8], salt: &[u8], udata: &[u8]) -> [u8; 32] {
    let mut first = Sha256::new();
    first.update(password);
    first.update(salt);
    first.update(udata);
    let mut k: Vec<u8> = first.finalize().to_vec();
    let mut round: i64 = 0;
    loop {
        let mut k1 = Vec::with_capacity(64 * (password.len() + k.len() + udata.len()));
        for _ in 0..64 {
            k1.extend_from_slice(password);
            k1.extend_from_slice(&k);
            k1.extend_from_slice(udata);
        }
        let e = aes128_cbc_nopad(&k[..16], &k[16..32], &k1);
        if e.len() < 16 {
            break;
        }
        // A big-endian number modulo 3 is the sum of its bytes modulo 3
        // (256 is 1 modulo 3).
        let m = e[..16].iter().map(|b| u32::from(*b)).sum::<u32>() % 3;
        k = match m {
            0 => Sha256::digest(&e).to_vec(),
            1 => Sha384::digest(&e).to_vec(),
            _ => Sha512::digest(&e).to_vec(),
        };
        round += 1;
        let last = i64::from(e.last().copied().unwrap_or(0));
        if round >= 64 && last <= round - 32 {
            break;
        }
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&k[..32]);
    out
}

/// Encrypts a file's strings and streams once its passwords are set up.
pub struct Encryptor {
    key: [u8; 32],
    random: Random,
    /// The /Encrypt dictionary.
    pub dictionary: String,
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2 + 2);
    s.push('<');
    for b in bytes {
        s.push_str(&format!("{b:02X}"));
    }
    s.push('>');
    s
}

impl Encryptor {
    /// Set up the file key and the /Encrypt dictionary for `security`;
    /// `random` fills buffers with random bytes.
    pub fn new(security: &Security, mut random: Random) -> Encryptor {
        let mut key = [0u8; 32];
        random(&mut key);
        let user = prepare(&security.user_password);
        let owner = if security.owner_password.is_empty() {
            user.clone()
        } else {
            prepare(&security.owner_password)
        };
        let zero_iv = [0u8; 16];
        // /U and /UE: the user password's check and the key it unwraps.
        let mut salts = [0u8; 16];
        random(&mut salts);
        let (validation, key_salt) = salts.split_at(8);
        let mut u = hash_r6(&user, validation, &[]).to_vec();
        u.extend_from_slice(validation);
        u.extend_from_slice(key_salt);
        let ue = aes256_cbc_nopad(&hash_r6(&user, key_salt, &[]), &zero_iv, &key);
        // /O and /OE: the same for the owner password, bound to /U.
        let mut salts = [0u8; 16];
        random(&mut salts);
        let (validation, key_salt) = salts.split_at(8);
        let mut o = hash_r6(&owner, validation, &u).to_vec();
        o.extend_from_slice(validation);
        o.extend_from_slice(key_salt);
        let oe = aes256_cbc_nopad(&hash_r6(&owner, key_salt, &u), &zero_iv, &key);
        // /Perms: the permissions again, encrypted with the file key.
        let p = security.permissions.p_value();
        let mut perms = [0u8; 16];
        perms[..4].copy_from_slice(&p.to_le_bytes());
        perms[4..8].copy_from_slice(&[0xFF; 4]);
        perms[8] = b'T';
        perms[9..12].copy_from_slice(b"adb");
        random(&mut perms[12..16]);
        let mut block = aes::cipher::generic_array::GenericArray::clone_from_slice(&perms);
        if let Ok(cipher) = aes::Aes256::new_from_slice(&key) {
            cipher.encrypt_block(&mut block);
        }
        // No top-level /Length: the standard allows it only with V 2 or 3
        // and some readers refuse a 256-bit value there. The crypt filter
        // carries the key length in bytes.
        let dictionary = format!(
            "<< /Filter /Standard /V 5 /R 6 /CF << /StdCF << /AuthEvent /DocOpen /CFM /AESV3 /Length 32 >> >> /StmF /StdCF /StrF /StdCF /O {} /U {} /OE {} /UE {} /P {p} /Perms {} /EncryptMetadata true >>",
            hex(&o),
            hex(&u),
            hex(&oe),
            hex(&ue),
            hex(&block)
        );
        Encryptor {
            key,
            random,
            dictionary,
        }
    }

    /// AES-256-CBC with a fresh IV in front and PKCS #7 padding.
    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        let mut iv = [0u8; 16];
        (self.random)(&mut iv);
        let mut buf = data.to_vec();
        let len = buf.len();
        buf.resize(len + 16, 0);
        let body = match cbc::Encryptor::<aes::Aes256>::new_from_slices(&self.key, &iv) {
            Ok(enc) => match enc.encrypt_padded_mut::<Pkcs7>(&mut buf, len) {
                Ok(out) => out.to_vec(),
                Err(_) => Vec::new(),
            },
            Err(_) => Vec::new(),
        };
        let mut out = iv.to_vec();
        out.extend_from_slice(&body);
        out
    }

    /// Encrypt one object's body as the writer made it: its strings (as
    /// hexadecimal strings) and, for a stream, its data (with /Length
    /// updated).
    pub fn encrypt_object(&mut self, body: &[u8]) -> Vec<u8> {
        const START: &[u8] = b">>\nstream\n";
        const END: &[u8] = b"\nendstream";
        if let Some(at) = find(body, START) {
            if body.ends_with(END) && at + START.len() <= body.len() - END.len() {
                let dict = &body[..at + 2];
                let data = &body[at + START.len()..body.len() - END.len()];
                let encrypted = self.encrypt(data);
                let dict = self.encrypt_strings(dict);
                let dict = set_length(&dict, encrypted.len());
                let mut out = dict;
                out.extend_from_slice(b"\nstream\n");
                out.extend_from_slice(&encrypted);
                out.extend_from_slice(END);
                return out;
            }
        }
        self.encrypt_strings(body)
    }

    /// Every literal and hexadecimal string of `text`, encrypted and
    /// written as a hexadecimal string.
    fn encrypt_strings(&mut self, text: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(text.len() + 64);
        let mut i = 0;
        while i < text.len() {
            match text[i] {
                b'(' => {
                    let (raw, next) = literal(text, i);
                    let enc = self.encrypt(&raw);
                    out.extend_from_slice(hex(&enc).as_bytes());
                    i = next;
                }
                b'<' if text.get(i + 1) != Some(&b'<') => {
                    let end = text[i..]
                        .iter()
                        .position(|b| *b == b'>')
                        .map(|p| i + p)
                        .unwrap_or(text.len());
                    let raw = unhex(&text[i + 1..end.min(text.len())]);
                    let enc = self.encrypt(&raw);
                    out.extend_from_slice(hex(&enc).as_bytes());
                    i = end + 1;
                }
                b'<' => {
                    out.extend_from_slice(b"<<");
                    i += 2;
                }
                c => {
                    out.push(c);
                    i += 1;
                }
            }
        }
        out
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The value of the literal string starting at `text[start]` (a `(`) and
/// the index after its closing parenthesis.
fn literal(text: &[u8], start: usize) -> (Vec<u8>, usize) {
    let mut raw = Vec::new();
    let mut depth = 0usize;
    let mut i = start;
    while i < text.len() {
        let c = text[i];
        match c {
            b'(' => {
                if depth > 0 {
                    raw.push(c);
                }
                depth += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return (raw, i + 1);
                }
                raw.push(c);
            }
            b'\\' => {
                i += 1;
                let Some(&e) = text.get(i) else { break };
                match e {
                    b'n' => raw.push(b'\n'),
                    b'r' => raw.push(b'\r'),
                    b't' => raw.push(b'\t'),
                    b'b' => raw.push(8),
                    b'f' => raw.push(12),
                    b'0'..=b'7' => {
                        let mut v = u32::from(e - b'0');
                        let mut n = 1;
                        while n < 3 {
                            match text.get(i + 1) {
                                Some(d @ b'0'..=b'7') => {
                                    v = v * 8 + u32::from(d - b'0');
                                    i += 1;
                                    n += 1;
                                }
                                _ => break,
                            }
                        }
                        raw.push((v & 0xFF) as u8);
                    }
                    b'\r' => {
                        if text.get(i + 1) == Some(&b'\n') {
                            i += 1;
                        }
                    }
                    b'\n' => {}
                    other => raw.push(other),
                }
            }
            _ => raw.push(c),
        }
        i += 1;
    }
    (raw, text.len())
}

fn unhex(digits: &[u8]) -> Vec<u8> {
    let vals: Vec<u8> = digits
        .iter()
        .filter_map(|c| (*c as char).to_digit(16).map(|v| v as u8))
        .collect();
    vals.chunks(2)
        .map(|p| (p[0] << 4) | p.get(1).copied().unwrap_or(0))
        .collect()
}

/// `dict` with its /Length set to `len`.
fn set_length(dict: &[u8], len: usize) -> Vec<u8> {
    let text = String::from_utf8_lossy(dict).to_string();
    if let Some(at) = text.rfind("/Length ") {
        let rest = &text[at + 8..];
        let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        format!("{}/Length {len}{}", &text[..at], &rest[digits..]).into_bytes()
    } else {
        dict.to_vec()
    }
}

// ----- Reading protected files -----------------------------------------

/// How a protected file's strings or streams are encrypted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Not at all (the Identity crypt filter).
    None,
    Rc4,
    Aes128,
    Aes256,
}

/// Why a protected file cannot be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The password given (usually none) does not open it.
    NeedsPassword,
    /// A security handler, revision or entry this reader cannot use.
    Unsupported(String),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::NeedsPassword => write!(f, "the file needs a password to open"),
            OpenError::Unsupported(what) => write!(f, "unsupported protection: {what}"),
        }
    }
}

/// The padding of revisions 2 to 4: a password is cut or filled to 32
/// bytes with this string (ISO 32000-2, 7.6.4.3.2).
const PAD: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

/// RC4 of `data` with `key` (encryption and decryption are the same).
pub fn rc4(key: &[u8], data: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    let mut s: [u8; 256] = std::array::from_fn(|i| i as u8);
    let mut j: u8 = 0;
    for i in 0..256 {
        j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
        s.swap(i, usize::from(j));
    }
    let (mut i, mut j) = (0u8, 0u8);
    data.iter()
        .map(|b| {
            i = i.wrapping_add(1);
            j = j.wrapping_add(s[usize::from(i)]);
            s.swap(usize::from(i), usize::from(j));
            b ^ s[usize::from(s[usize::from(i)].wrapping_add(s[usize::from(j)]))]
        })
        .collect()
}

/// AES-CBC decryption of `data` (16 bytes of IV, then the blocks) with
/// PKCS #7 padding removed; data that is not a whole number of blocks
/// after its IV comes back as it was, the best a damaged file allows.
fn aes_cbc_decrypt<C>(key: &[u8], data: &[u8]) -> Vec<u8>
where
    C: aes::cipher::BlockCipher + aes::cipher::BlockDecrypt + aes::cipher::KeyInit,
{
    if data.len() < 16 {
        return Vec::new();
    }
    let (iv, body) = data.split_at(16);
    if body.len() % 16 != 0 {
        return data.to_vec();
    }
    let decryptor = || cbc::Decryptor::<C>::new_from_slices(key, iv).ok();
    let Some(dec) = decryptor() else {
        return data.to_vec();
    };
    let mut buf = body.to_vec();
    if let Ok(out) = dec.decrypt_padded_mut::<Pkcs7>(&mut buf) {
        return out.to_vec();
    }
    // Wrong padding: keep every block.
    let mut buf = body.to_vec();
    match decryptor().map(|d| {
        d.decrypt_padded_mut::<NoPadding>(&mut buf)
            .map(<[u8]>::to_vec)
    }) {
        Some(Ok(out)) => out,
        _ => Vec::new(),
    }
}

/// AES-256-CBC without padding and a zero IV: how /UE and /OE wrap the
/// file key.
fn aes256_unwrap(key: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    let dec = cbc::Decryptor::<aes::Aes256>::new_from_slices(key, &[0u8; 16]).ok()?;
    let mut buf = data.to_vec();
    dec.decrypt_padded_mut::<NoPadding>(&mut buf)
        .ok()
        .map(<[u8]>::to_vec)
}

/// Decrypts the strings and streams of a protected file.
#[derive(Clone)]
pub struct Decryptor {
    key: Vec<u8>,
    strings: Method,
    streams: Method,
    encrypt_metadata: bool,
}

impl std::fmt::Debug for Decryptor {
    // The key stays out of any log.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decryptor")
            .field("strings", &self.strings)
            .field("streams", &self.streams)
            .field("encrypt_metadata", &self.encrypt_metadata)
            .finish_non_exhaustive()
    }
}

impl Decryptor {
    /// Open a file protected by the standard security handler: `dict` is
    /// its /Encrypt dictionary, `id` the first string of the trailer's /ID
    /// and `password` the user or owner password (empty for a file that
    /// opens without one).
    pub fn open(dict: &Dictionary, id: &[u8], password: &str) -> Result<Decryptor, OpenError> {
        let name = |k: &[u8]| {
            dict.get(k)
                .and_then(Object::as_name)
                .ok()
                .map(<[u8]>::to_vec)
        };
        let int = |k: &[u8]| dict.get(k).and_then(Object::as_i64).ok();
        let bytes = |k: &[u8]| {
            dict.get(k)
                .and_then(Object::as_str)
                .map(<[u8]>::to_vec)
                .unwrap_or_default()
        };
        let unsupported = |what: String| Err(OpenError::Unsupported(what));
        if name(b"Filter").as_deref() != Some(b"Standard".as_slice()) {
            let handler = name(b"Filter").unwrap_or_default();
            return unsupported(format!(
                "security handler {}",
                String::from_utf8_lossy(&handler)
            ));
        }
        let v = int(b"V").unwrap_or(0);
        let r = int(b"R").unwrap_or(0);
        // /P is a 32-bit value; some files write it unsigned.
        let p = int(b"P").unwrap_or(0) as u32;
        let encrypt_metadata = dict
            .get(b"EncryptMetadata")
            .and_then(Object::as_bool)
            .unwrap_or(true);
        let (strings, streams) = match v {
            1..=3 => (Method::Rc4, Method::Rc4),
            4 | 5 => {
                let filter = |entry: &[u8]| -> Result<Method, OpenError> {
                    // Absent entries mean the Identity filter.
                    let Some(n) = name(entry) else {
                        return Ok(Method::None);
                    };
                    if n == b"Identity" {
                        return Ok(Method::None);
                    }
                    let cfm = dict
                        .get(b"CF")
                        .and_then(Object::as_dict)
                        .and_then(|cf| cf.get(&n))
                        .and_then(Object::as_dict)
                        .and_then(|f| f.get(b"CFM"))
                        .and_then(Object::as_name)
                        .map(<[u8]>::to_vec)
                        .unwrap_or_default();
                    match cfm.as_slice() {
                        b"None" | b"" => Ok(Method::None),
                        b"V2" => Ok(Method::Rc4),
                        b"AESV2" => Ok(Method::Aes128),
                        b"AESV3" => Ok(Method::Aes256),
                        other => Err(OpenError::Unsupported(format!(
                            "crypt filter {}",
                            String::from_utf8_lossy(other)
                        ))),
                    }
                };
                (filter(b"StrF")?, filter(b"StmF")?)
            }
            other => return unsupported(format!("version {other}")),
        };
        let (o, u) = (bytes(b"O"), bytes(b"U"));
        let key = match r {
            2..=4 => {
                if o.len() < 32 || u.len() < 16 {
                    return unsupported("damaged password entries".into());
                }
                // Key length in bytes: 5 for revision 2, else /Length bits.
                let n = if r == 2 {
                    5
                } else {
                    match int(b"Length") {
                        Some(bits) if (40..=128).contains(&bits) => (bits / 8) as usize,
                        _ if v >= 4 => 16,
                        _ => 5,
                    }
                };
                let mut padded = prepare(password);
                padded.truncate(32);
                let fill = 32 - padded.len();
                padded.extend_from_slice(&PAD[..fill]);
                // Algorithm 2: the key from the password, /O, /P and the ID.
                let mut md5 = Md5::new();
                md5.update(&padded);
                md5.update(&o[..32]);
                md5.update(p.to_le_bytes());
                md5.update(id);
                if r >= 4 && !encrypt_metadata {
                    md5.update([0xFF; 4]);
                }
                let mut hash = md5.finalize().to_vec();
                if r >= 3 {
                    for _ in 0..50 {
                        hash = Md5::digest(&hash[..n]).to_vec();
                    }
                }
                let key = hash[..n].to_vec();
                // Algorithms 4 and 5: the user password's check value.
                let check = if r == 2 {
                    rc4(&key, &PAD) == u[..32.min(u.len())]
                } else {
                    let mut md5 = Md5::new();
                    md5.update(PAD);
                    md5.update(id);
                    let mut x = rc4(&key, &md5.finalize());
                    for i in 1..=19u8 {
                        let k: Vec<u8> = key.iter().map(|b| b ^ i).collect();
                        x = rc4(&k, &x);
                    }
                    x[..16] == u[..16]
                };
                if !check {
                    return Err(OpenError::NeedsPassword);
                }
                key
            }
            5 | 6 => {
                let (oe, ue) = (bytes(b"OE"), bytes(b"UE"));
                if o.len() < 48 || u.len() < 48 || oe.len() < 32 || ue.len() < 32 {
                    return unsupported("damaged password entries".into());
                }
                let pw = prepare(password);
                let hash = |salt: &[u8], udata: &[u8]| -> [u8; 32] {
                    if r == 5 {
                        let mut h = Sha256::new();
                        h.update(&pw);
                        h.update(salt);
                        h.update(udata);
                        h.finalize().into()
                    } else {
                        hash_r6(&pw, salt, udata)
                    }
                };
                // Algorithm 2.A: the owner password first, then the user's.
                let key = if hash(&o[32..40], &u[..48]) == o[..32] {
                    aes256_unwrap(&hash(&o[40..48], &u[..48]), &oe[..32])
                } else if hash(&u[32..40], &[]) == u[..32] {
                    aes256_unwrap(&hash(&u[40..48], &[]), &ue[..32])
                } else {
                    return Err(OpenError::NeedsPassword);
                };
                match key {
                    Some(k) if k.len() == 32 => k,
                    _ => return unsupported("damaged key entries".into()),
                }
            }
            other => return unsupported(format!("revision {other}")),
        };
        Ok(Decryptor {
            key,
            strings,
            streams,
            encrypt_metadata,
        })
    }

    /// The key for one object (algorithm 1): AES-256 uses the file key as
    /// it is; RC4 and AES-128 mix in the object's number and generation.
    fn object_key(&self, id: ObjectId, method: Method) -> Vec<u8> {
        if method == Method::Aes256 {
            return self.key.clone();
        }
        let mut md5 = Md5::new();
        md5.update(&self.key);
        md5.update(&id.0.to_le_bytes()[..3]);
        md5.update(id.1.to_le_bytes());
        if method == Method::Aes128 {
            md5.update(b"sAlT");
        }
        let hash = md5.finalize();
        hash[..(self.key.len() + 5).min(16)].to_vec()
    }

    fn decrypt_bytes(&self, id: ObjectId, method: Method, data: &[u8]) -> Vec<u8> {
        match method {
            Method::None => data.to_vec(),
            Method::Rc4 => rc4(&self.object_key(id, method), data),
            Method::Aes128 => aes_cbc_decrypt::<aes::Aes128>(&self.object_key(id, method), data),
            Method::Aes256 => aes_cbc_decrypt::<aes::Aes256>(&self.key, data),
        }
    }

    /// Decrypt the strings of indirect object `id` and, for a stream, its
    /// data. Cross-reference streams, unencrypted metadata and streams
    /// with their own crypt filter are left as they are.
    pub fn decrypt_object(&self, id: ObjectId, object: &mut Object) {
        match object {
            Object::String(bytes, _) => *bytes = self.decrypt_bytes(id, self.strings, bytes),
            Object::Array(items) => {
                for item in items {
                    self.decrypt_object(id, item);
                }
            }
            Object::Dictionary(dict) => self.decrypt_dictionary(id, dict),
            Object::Stream(stream) => {
                if stream.dict.has_type(b"XRef") {
                    return;
                }
                self.decrypt_dictionary(id, &mut stream.dict);
                self.decrypt_stream_data(id, stream);
            }
            _ => {}
        }
    }

    /// Decrypt a stream's data only (its dictionary is done already).
    pub fn decrypt_stream_data(&self, id: ObjectId, stream: &mut lopdf::Stream) {
        let dict = &stream.dict;
        if dict.has_type(b"XRef") || (dict.has_type(b"Metadata") && !self.encrypt_metadata) {
            return;
        }
        let own_filter = match dict.get(b"Filter") {
            Ok(Object::Name(n)) => n == b"Crypt",
            Ok(Object::Array(a)) => a.first().and_then(|o| o.as_name().ok()) == Some(b"Crypt"),
            _ => false,
        };
        if own_filter {
            return;
        }
        let data = self.decrypt_bytes(id, self.streams, &stream.content);
        stream.set_content(data);
    }

    fn decrypt_dictionary(&self, id: ObjectId, dict: &mut Dictionary) {
        // A signature's /Contents is never encrypted.
        let signature = dict.has(b"ByteRange");
        for (key, value) in dict.iter_mut() {
            if signature && key == b"Contents" {
                continue;
            }
            self.decrypt_object(id, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed stream of bytes in place of randomness, for repeatable tests.
    pub(crate) fn counter() -> Random {
        let mut n: u8 = 0;
        Box::new(move |buf: &mut [u8]| {
            for b in buf.iter_mut() {
                n = n.wrapping_mul(31).wrapping_add(17);
                *b = n;
            }
        })
    }

    #[test]
    fn permissions_make_the_p_value() {
        assert_eq!(Permissions::default().p_value(), -4);
        let none = Permissions {
            print: false,
            print_high_quality: false,
            copy: false,
            modify: false,
            annotate: false,
            fill_forms: false,
            assemble: false,
            accessibility: false,
        };
        assert_eq!(none.p_value(), -3904);
        // Printing without high quality: bit 3 only.
        let low = Permissions {
            print_high_quality: false,
            ..none
        };
        assert_eq!(low.p_value(), -3904);
        let low = Permissions { print: true, ..low };
        assert_eq!(low.p_value(), -3900);
    }

    #[test]
    fn literal_strings_decode_their_escapes() {
        let (raw, next) = literal(b"(a\\(b\\)c\\\\d\\101(e)) rest", 0);
        assert_eq!(raw, b"a(b)c\\dA(e)");
        assert_eq!(next, 19);
        assert_eq!(unhex(b"4142 43"), b"ABC");
        assert_eq!(unhex(b"414"), b"A@");
        assert_eq!(
            set_length(b"<< /Length 12 /Filter /X >>", 48),
            b"<< /Length 48 /Filter /X >>"
        );
    }

    #[test]
    fn rc4_matches_the_published_vectors() {
        assert_eq!(rc4(b"Key", b"Plaintext"), unhex(b"BBF316E8D940AF0AD3"));
        assert_eq!(rc4(b"Wiki", b"pedia"), unhex(b"1021BF0420"));
        assert_eq!(
            rc4(b"Secret", b"Attack at dawn"),
            unhex(b"45A01F645FC35B383552544B9BF5")
        );
    }

    #[test]
    fn what_is_encrypted_decrypts_again() {
        let mut e = Encryptor::new(
            &Security {
                user_password: String::new(),
                owner_password: "owner".into(),
                permissions: Permissions::default(),
            },
            counter(),
        );
        let d = Decryptor {
            key: e.key.to_vec(),
            strings: Method::Aes256,
            streams: Method::Aes256,
            encrypt_metadata: true,
        };
        let mut s = Object::string_literal(e.encrypt(b"Secret plan"));
        d.decrypt_object((3, 0), &mut s);
        assert_eq!(s.as_str().ok(), Some(b"Secret plan".as_slice()));
        // Every length, padding included.
        for n in [0usize, 1, 15, 16, 17, 100] {
            let data = vec![b'x'; n];
            let mut stream =
                Object::Stream(lopdf::Stream::new(Dictionary::new(), e.encrypt(&data)));
            d.decrypt_object((4, 0), &mut stream);
            assert_eq!(
                stream.as_stream().map(|s| s.content.clone()).ok(),
                Some(data)
            );
        }
        // Cross-reference streams and signature contents are left alone.
        let mut xref = Dictionary::new();
        xref.set("Type", "XRef");
        let mut stream = Object::Stream(lopdf::Stream::new(xref, b"raw".to_vec()));
        d.decrypt_object((5, 0), &mut stream);
        assert_eq!(
            stream.as_stream().map(|s| s.content.clone()).ok(),
            Some(b"raw".to_vec())
        );
        let mut sig = Dictionary::new();
        sig.set("ByteRange", Object::Array(vec![]));
        sig.set("Contents", Object::string_literal(b"raw".to_vec()));
        let mut sig = Object::Dictionary(sig);
        d.decrypt_object((6, 0), &mut sig);
        let contents = sig
            .as_dict()
            .and_then(|d| d.get(b"Contents"))
            .and_then(Object::as_str);
        assert_eq!(contents.ok(), Some(b"raw".as_slice()));
    }

    #[test]
    fn revision_5_files_open_with_either_password() {
        let key = [7u8; 32];
        let sha = |parts: &[&[u8]]| {
            let mut h = Sha256::new();
            for p in parts {
                h.update(p);
            }
            h.finalize().to_vec()
        };
        let (vs, ks, ovs, oks) = ([1u8; 8], [2u8; 8], [3u8; 8], [4u8; 8]);
        let mut u = sha(&[b"", &vs]);
        u.extend_from_slice(&vs);
        u.extend_from_slice(&ks);
        let ue = aes256_cbc_nopad(&sha(&[b"", &ks]), &[0; 16], &key);
        let mut o = sha(&[b"owner", &ovs, &u]);
        o.extend_from_slice(&ovs);
        o.extend_from_slice(&oks);
        let oe = aes256_cbc_nopad(&sha(&[b"owner", &oks, &u]), &[0; 16], &key);
        let mut filter = Dictionary::new();
        filter.set("CFM", "AESV3");
        let mut cf = Dictionary::new();
        cf.set("StdCF", filter);
        let mut dict = Dictionary::new();
        dict.set("Filter", "Standard");
        dict.set("V", 5);
        dict.set("R", 5);
        dict.set("P", -4);
        dict.set("O", Object::string_literal(o));
        dict.set("U", Object::string_literal(u));
        dict.set("OE", Object::string_literal(oe));
        dict.set("UE", Object::string_literal(ue));
        dict.set("CF", cf);
        dict.set("StmF", "StdCF");
        dict.set("StrF", "StdCF");
        for password in ["", "owner"] {
            let d = Decryptor::open(&dict, b"", password).expect("opens");
            assert_eq!(d.key, key);
            assert_eq!((d.strings, d.streams), (Method::Aes256, Method::Aes256));
        }
        assert_eq!(
            Decryptor::open(&dict, b"", "nope").map(|_| ()),
            Err(OpenError::NeedsPassword)
        );
        // No security handler of ours.
        dict.set("Filter", "Other");
        assert!(matches!(
            Decryptor::open(&dict, b"", ""),
            Err(OpenError::Unsupported(_))
        ));
    }

    #[test]
    fn objects_get_their_strings_and_streams_encrypted() {
        let mut e = Encryptor::new(
            &Security {
                user_password: "user".into(),
                owner_password: "owner".into(),
                permissions: Permissions::default(),
            },
            counter(),
        );
        assert!(e.dictionary.contains("/R 6") && e.dictionary.contains("/P -4"));
        let body = e.encrypt_object(b"<< /Title (Hello) /Kids [<0102>] /Size 3 >>");
        let text = String::from_utf8_lossy(&body);
        assert!(!text.contains("Hello") && text.contains("/Size 3"));
        // IV (16) plus one padded block (16) each: 64 hex digits.
        assert_eq!(text.matches('<').count() - 2, 2, "{text}");
        let stream = e.encrypt_object(b"<< /Length 5 >>\nstream\nhello\nendstream");
        assert!(stream.starts_with(b"<< /Length 32 >>\nstream\n"));
        assert_eq!(
            stream.len(),
            b"<< /Length 32 >>\nstream\n".len() + 32 + b"\nendstream".len()
        );
    }
}
