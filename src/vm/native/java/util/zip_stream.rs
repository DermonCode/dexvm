//! java.util.zip.ZipInputStream / ZipEntry host shims: this VM has no real
//! streaming, so the whole archive is decoded eagerly into (name, content)
//! pairs on construction, and `getNextEntry` selects the readable entry.

use crate::vm::native::*;
use std::io::{Cursor, Read as _};

fn zip_input_stream_init(vm: &mut Vm, args: &[JValue]) -> R {
    let bytes = match payload(vm, args[1]) {
        Some(Native::ByteArrayInputStream { bytes, pos }) => bytes[*pos..].to_vec(),
        _ => return Err(npe(vm)),
    };
    let entries = match zip::ZipArchive::new(Cursor::new(bytes)) {
        Ok(mut archive) => {
            let mut out = Vec::new();
            for i in 0..archive.len() {
                if let Ok(mut file) = archive.by_index(i) {
                    let name = file.name().to_string();
                    let mut content = Vec::new();
                    let _ = file.read_to_end(&mut content);
                    out.push((name, content));
                }
            }
            out
        }
        Err(_) => Vec::new(),
    };
    let JValue::Obj(id) = args[0] else {
        return Err(npe(vm));
    };
    vm.arena.objects[id as usize].native = Some(Native::ZipReader { entries, idx: -1, pos: 0 });
    Ok(JValue::Null)
}

fn zip_input_stream_get_next_entry(vm: &mut Vm, args: &[JValue]) -> R {
    let (name, done) = match payload_mut(vm, args[0]) {
        Some(Native::ZipReader { entries, idx, pos }) => {
            *idx += 1;
            *pos = 0;
            match entries.get(*idx as usize) {
                Some((name, _)) => (name.clone(), false),
                None => (String::new(), true),
            }
        }
        _ => return Err(npe(vm)),
    };
    if done {
        return Ok(JValue::Null);
    }
    alloc(vm, "Ljava/util/zip/ZipEntry;", Native::ZipEntryName(name))
}

fn zip_input_stream_read(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::ZipReader { entries, idx, pos }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    let Some((_, bytes)) = usize::try_from(*idx).ok().and_then(|idx| entries.get(idx)) else {
        return Ok(JValue::Int(-1));
    };
    if *pos >= bytes.len() {
        return Ok(JValue::Int(-1));
    }
    let byte = bytes[*pos];
    *pos += 1;
    Ok(JValue::Int(i32::from(byte)))
}

fn zip_input_stream_read_buf(vm: &mut Vm, args: &[JValue]) -> R {
    let off = int_of(vm, args[2]);
    let len = int_of(vm, args[3]);
    let capacity = match payload(vm, args[1]) {
        Some(Native::Array(ArrayData::Byte(dst))) => dst.len(),
        _ => return Err(npe(vm)),
    };
    if off < 0 || len < 0 || (off as usize) > capacity || (len as usize) > capacity - off as usize {
        return Err(iae(vm, "Invalid read range"));
    }
    let (src, dst) = payload_mut_two(vm, args[0], args[1]);
    let (Some(Native::ZipReader { entries, idx, pos }), Some(Native::Array(ArrayData::Byte(dst)))) = (src, dst) else {
        return Err(npe(vm));
    };
    if len == 0 {
        return Ok(JValue::Int(0));
    }
    let Some((_, bytes)) = usize::try_from(*idx).ok().and_then(|idx| entries.get(idx)) else {
        return Ok(JValue::Int(-1));
    };
    if *pos >= bytes.len() {
        return Ok(JValue::Int(-1));
    }
    let count = (len as usize).min(bytes.len() - *pos);
    for (out, byte) in dst[off as usize..off as usize + count].iter_mut().zip(&bytes[*pos..*pos + count]) {
        *out = *byte as i8;
    }
    *pos += count;
    Ok(JValue::Int(count as i32))
}

fn zip_input_stream_read_all_buf(vm: &mut Vm, args: &[JValue]) -> R {
    let len = match payload(vm, args[1]) {
        Some(Native::Array(ArrayData::Byte(dst))) => dst.len() as i32,
        _ => return Err(npe(vm)),
    };
    zip_input_stream_read_buf(vm, &[args[0], args[1], JValue::Int(0), JValue::Int(len)])
}

fn zip_input_stream_close_entry(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::ZipReader { entries, idx, pos }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    if let Some((_, bytes)) = usize::try_from(*idx).ok().and_then(|idx| entries.get(idx)) {
        *pos = bytes.len();
    }
    Ok(JValue::Null)
}

fn zip_input_stream_close(vm: &mut Vm, args: &[JValue]) -> R {
    let Some(Native::ZipReader { entries, idx, .. }) = payload_mut(vm, args[0]) else {
        return Err(npe(vm));
    };
    *idx = entries.len() as i32;
    Ok(JValue::Null)
}

fn zip_entry_get_name(vm: &mut Vm, args: &[JValue]) -> R {
    let name = match payload(vm, args[0]) {
        Some(Native::ZipEntryName(name)) => name.clone(),
        _ => return Err(npe(vm)),
    };
    Ok(new_str(vm, &name))
}

pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/util/zip/ZipInputStream;",
        "<init>",
        "(Ljava/io/InputStream;)V",
        true,
        zip_input_stream_init
    ),
    ne!(
        "Ljava/util/zip/ZipInputStream;",
        "getNextEntry",
        "()Ljava/util/zip/ZipEntry;",
        true,
        zip_input_stream_get_next_entry
    ),
    ne!("Ljava/util/zip/ZipInputStream;", "read", "()I", true, zip_input_stream_read),
    ne!("Ljava/util/zip/ZipInputStream;", "read", "([B)I", true, zip_input_stream_read_all_buf),
    ne!("Ljava/util/zip/ZipInputStream;", "read", "([BII)I", true, zip_input_stream_read_buf),
    ne!(
        "Ljava/util/zip/ZipInputStream;",
        "closeEntry",
        "()V",
        true,
        zip_input_stream_close_entry
    ),
    ne!(
        "Ljava/util/zip/ZipInputStream;",
        "close",
        "()V",
        true,
        zip_input_stream_close
    ),
    ne!(
        "Ljava/util/zip/ZipEntry;",
        "getName",
        "()Ljava/lang/String;",
        true,
        zip_entry_get_name
    ),
];
