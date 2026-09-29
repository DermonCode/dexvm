//! java.net.URLDecoder host shims.

use crate::vm::native::*;

fn url_decoder_decode(vm: &mut Vm, args: &[JValue]) -> R {
    let value = jstr(vm, args[0])?;
    Ok(new_str(vm, &super::form::decode(&value)))
}

pub(crate) const TABLE: &[NativeEntry] = &[ne!(
    "Ljava/net/URLDecoder;",
    "decode",
    "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
    false,
    url_decoder_decode
)];
