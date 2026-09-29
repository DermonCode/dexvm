//! java.net.URLEncoder host shims.

use crate::vm::native::*;

fn url_encoder_encode(vm: &mut Vm, args: &[JValue]) -> R {
    let value = jstr(vm, args[0])?;
    Ok(new_str(vm, &super::form::encode(&value)))
}

pub(crate) const TABLE: &[NativeEntry] = &[ne!(
    "Ljava/net/URLEncoder;",
    "encode",
    "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
    false,
    url_encoder_encode
)];
