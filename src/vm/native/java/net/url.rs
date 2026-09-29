//! java.net.URL host shims.

use crate::vm::native::*;

fn url_init(vm: &mut Vm, args: &[JValue]) -> R {
    let value = jstr(vm, args[1])?;
    let JValue::Obj(id) = args[0] else {
        return Err(npe(vm));
    };
    vm.arena.objects[id as usize].native = Some(Native::URI(value));
    Ok(JValue::Null)
}

fn url_get_host(vm: &mut Vm, args: &[JValue]) -> R {
    let value = match payload(vm, args[0]) {
        Some(Native::URI(value)) => value.clone(),
        _ => return Err(npe(vm)),
    };
    let authority = value
        .split("://")
        .last()
        .and_then(|part| part.strip_prefix("//").or(Some(part)))
        .unwrap_or(&value);
    Ok(new_str(
        vm,
        authority.split(['/', ':']).next().unwrap_or(""),
    ))
}

fn url_get_path(vm: &mut Vm, args: &[JValue]) -> R {
    let value = match payload(vm, args[0]) {
        Some(Native::URI(value)) => value.clone(),
        _ => return Err(npe(vm)),
    };
    let after_scheme = value.split("://").last().unwrap_or(&value);
    let path = if let Some(rest) = after_scheme.strip_prefix("//") {
        rest.find('/').map(|i| &rest[i..]).unwrap_or("")
    } else {
        after_scheme
    };
    Ok(new_str(vm, path.split(['?', '#']).next().unwrap_or("")))
}

fn url_open_stream(vm: &mut Vm, args: &[JValue]) -> R {
    let key = match payload(vm, args[0]) {
        Some(Native::URI(value)) => value.strip_prefix("resource:/").map(str::to_owned),
        _ => None,
    };
    let Some(bytes) = key.and_then(|key| vm.resources.get(&key).cloned()) else {
        return Err(iae(vm, "URL resource unavailable"));
    };
    alloc(vm, "Ljava/io/ByteArrayInputStream;", Native::ByteArrayInputStream { bytes, pos: 0 })
}

pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/net/URL;",
        "openStream",
        "()Ljava/io/InputStream;",
        true,
        url_open_stream
    ),
    ne!(
        "Ljava/net/URL;",
        "<init>",
        "(Ljava/lang/String;)V",
        true,
        url_init
    ),
    ne!(
        "Ljava/net/URL;",
        "getHost",
        "()Ljava/lang/String;",
        true,
        url_get_host
    ),
    ne!(
        "Ljava/net/URL;",
        "getPath",
        "()Ljava/lang/String;",
        true,
        url_get_path
    ),
];
