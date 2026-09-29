//! java.nio.charset.Charset host shims.

use crate::vm::native::*;

pub(crate) fn lazy_charset(vm: &mut Vm, name: &str) -> JValue {
    let Ok(class) = vm.ensure_class_by_desc("Ljava/nio/charset/Charset;") else {
        return JValue::Null;
    };
    JValue::Obj(
        vm.arena
            .alloc(class, Vec::new(), Some(Native::Str(name.to_string()))),
    )
}

pub fn lazy_charset_utf8(vm: &mut Vm) -> JValue {
    lazy_charset(vm, "UTF-8")
}
pub fn lazy_charset_iso(vm: &mut Vm) -> JValue {
    lazy_charset(vm, "ISO-8859-1")
}
pub fn lazy_charset_ascii(vm: &mut Vm) -> JValue {
    lazy_charset(vm, "US-ASCII")
}

// java.nio.charset.Charset
// ---------------------------------------------------------------------------

pub(crate) fn normalize_charset(name: &str) -> Option<String> {
    let up = name.trim().to_uppercase();
    let n = match up.as_str() {
        "UTF-8" | "UTF8" | "UTF_8" => "UTF-8",
        "US-ASCII" | "ASCII" | "US_ASCII" | "646" => "US-ASCII",
        "UTF-16" | "UTF16" | "UTF_16" => "UTF-16",
        "UTF-16LE" | "UTF16LE" | "UTF_16LE" => "UTF-16LE",
        "UTF-16BE" | "UTF16BE" | "UTF_16BE" => "UTF-16BE",
        "UTF-32" | "UTF32" | "UTF_32" => "UTF-32",
        "WINDOWS-1252" | "WINDOWS1252" | "CP1252" => "WINDOWS-1252",
        "ISO-8859-1" | "ISO8859-1" | "ISO_8859-1" | "ISO8859_1" | "LATIN1" | "L1" | "8859-1"
        | "CP819" => "ISO-8859-1",
        _ => return None,
    };
    Some(n.to_string())
}

/// Decodes the charsets advertised by the VM's Charset shim.
pub(crate) fn decode_charset(bytes: &[u8], name: &str) -> Option<String> {
    let charset = normalize_charset(name)?;
    Some(match charset.as_str() {
        "UTF-8" => String::from_utf8_lossy(bytes).into_owned(),
        "US-ASCII" => bytes.iter().map(|byte| if *byte < 128 { char::from(*byte) } else { '\u{fffd}' }).collect(),
        "ISO-8859-1" => bytes.iter().map(|byte| char::from(*byte)).collect(),
        "WINDOWS-1252" => bytes.iter().map(|byte| {
            const UPPER: [char; 32] = [
                '€', '\u{0081}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{008d}', 'Ž', '\u{008f}',
                '\u{0090}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{009d}', 'ž', 'Ÿ',
            ];
            if (0x80..=0x9f).contains(byte) { UPPER[(*byte - 0x80) as usize] } else { char::from(*byte) }
        }).collect(),
        "UTF-16" | "UTF-16LE" | "UTF-16BE" => {
            let (little, data) = if charset == "UTF-16" {
                match bytes {
                    [0xff, 0xfe, rest @ ..] => (true, rest),
                    [0xfe, 0xff, rest @ ..] => (false, rest),
                    _ => (false, bytes),
                }
            } else { (charset == "UTF-16LE", bytes) };
            let words: Vec<u16> = data.chunks_exact(2).map(|pair| {
                if little { u16::from_le_bytes([pair[0], pair[1]]) }
                else { u16::from_be_bytes([pair[0], pair[1]]) }
            }).collect();
            let mut text = String::from_utf16_lossy(&words);
            if data.len() % 2 != 0 { text.push('\u{fffd}'); }
            text
        }
        "UTF-32" => {
            let (little, data) = match bytes {
                [0xff, 0xfe, 0, 0, rest @ ..] => (true, rest),
                [0, 0, 0xfe, 0xff, rest @ ..] => (false, rest),
                _ => (false, bytes),
            };
            let mut text: String = data.chunks_exact(4).map(|word| {
                let word = [word[0], word[1], word[2], word[3]];
                char::from_u32(if little { u32::from_le_bytes(word) } else { u32::from_be_bytes(word) }).unwrap_or('\u{fffd}')
            }).collect();
            if data.len() % 4 != 0 { text.push('\u{fffd}'); }
            text
        }
        _ => return None,
    })
}

pub(crate) fn charset_for_name(vm: &mut Vm, args: &[JValue]) -> R {
    let name = jstr(vm, args[0])?;
    match normalize_charset(&name) {
        Some(n) => alloc(vm, "Ljava/nio/charset/Charset;", Native::Str(n)),
        None => Err(iae(vm, format!("Unsupported charset: {name}"))),
    }
}

pub(crate) fn charset_name(vm: &mut Vm, args: &[JValue]) -> R {
    let name = match payload(vm, args[0]) {
        Some(Native::Str(s)) => s.clone(),
        _ => return Err(npe(vm)),
    };
    Ok(new_str(vm, &name))
}

pub(crate) fn charset_can_encode(_vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(JValue::Int(1))
}

pub(crate) fn charset_default_charset(vm: &mut Vm, _args: &[JValue]) -> R {
    Ok(lazy_charset_utf8(vm))
}

pub(crate) fn charset_is_supported(vm: &mut Vm, args: &[JValue]) -> R {
    let name = jstr(vm, args[0])?;
    Ok(JValue::Int(i32::from(normalize_charset(&name).is_some())))
}

pub(crate) fn charset_new_decoder(vm: &mut Vm, args: &[JValue]) -> R {
    let name = jstr(vm, args[0]).unwrap_or_default();
    alloc(vm, "Ljava/nio/charset/CharsetDecoder;", Native::Str(name))
}
pub(crate) fn charset_new_encoder(vm: &mut Vm, args: &[JValue]) -> R {
    let name = jstr(vm, args[0]).unwrap_or_default();
    alloc(vm, "Ljava/nio/charset/CharsetEncoder;", Native::Str(name))
}
pub(crate) fn codec_charset(vm: &mut Vm, args: &[JValue]) -> R {
    let name = match payload(vm, args[0]) {
        Some(Native::Str(s)) => s.clone(),
        _ => return Err(npe(vm)),
    };
    alloc(vm, "Ljava/nio/charset/Charset;", Native::Str(name))
}

/// Native methods for Ljava/nio/charset/Charset;
pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/nio/charset/Charset;",
        "forName",
        "(Ljava/lang/String;)Ljava/nio/charset/Charset;",
        false,
        charset_for_name
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "name",
        "()Ljava/lang/String;",
        true,
        charset_name
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "toString",
        "()Ljava/lang/String;",
        true,
        charset_name
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "displayName",
        "()Ljava/lang/String;",
        true,
        charset_name
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "displayName",
        "(Ljava/util/Locale;)Ljava/lang/String;",
        true,
        charset_name
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "canEncode",
        "()Z",
        true,
        charset_can_encode
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "defaultCharset",
        "()Ljava/nio/charset/Charset;",
        false,
        charset_default_charset
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "isSupported",
        "(Ljava/lang/String;)Z",
        false,
        charset_is_supported
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "newDecoder",
        "()Ljava/nio/charset/CharsetDecoder;",
        true,
        charset_new_decoder
    ),
    ne!(
        "Ljava/nio/charset/Charset;",
        "newEncoder",
        "()Ljava/nio/charset/CharsetEncoder;",
        true,
        charset_new_encoder
    ),
    ne!(
        "Ljava/nio/charset/CharsetDecoder;",
        "charset",
        "()Ljava/nio/charset/Charset;",
        true,
        codec_charset
    ),
    ne!(
        "Ljava/nio/charset/CharsetEncoder;",
        "charset",
        "()Ljava/nio/charset/Charset;",
        true,
        codec_charset
    ),
];
