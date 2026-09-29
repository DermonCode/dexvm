//! java.net host shims.

use crate::vm::native::*;

mod inet_address;
mod uri;
mod url;
mod url_decoder;
mod url_encoder;
mod form;

/// All java.net native tables, grouped for `register`.
pub(crate) const NET_TABLE: &[&[NativeEntry]] = &[
    inet_address::TABLE,
    uri::TABLE,
    url::TABLE,
    url_decoder::TABLE,
    url_encoder::TABLE,
];
