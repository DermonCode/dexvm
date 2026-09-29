//! java.io.InputStreamReader host shims.

use crate::vm::native::*;

fn input_stream_reader_init(vm: &mut Vm, args: &[JValue]) -> R {
    let charset = match payload(vm, args[2]) {
        Some(Native::Str(name)) => name.clone(),
        _ => return Err(npe(vm)),
    };
    let bytes = match payload(vm, args[1]) {
        Some(Native::ByteArrayInputStream { bytes, pos }) => bytes[*pos..].to_vec(),
        _ => return Err(npe(vm)),
    };
    let text = decode_charset(&bytes, &charset)
        .ok_or_else(|| iae(vm, format!("Unsupported charset: {charset}")))?;
    let Some(JValue::Obj(this)) = args.first().copied() else {
        return Err(npe(vm));
    };
    vm.arena.objects[this as usize].native = Some(Native::Reader(text));
    Ok(JValue::Null)
}

pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/io/InputStreamReader;",
        "<init>",
        "(Ljava/io/InputStream;Ljava/lang/String;)V",
        true,
        input_stream_reader_init
    ),
    ne!(
        "Ljava/io/InputStreamReader;",
        "<init>",
        "(Ljava/io/InputStream;Ljava/nio/charset/Charset;)V",
        true,
        input_stream_reader_init
    ),
];
