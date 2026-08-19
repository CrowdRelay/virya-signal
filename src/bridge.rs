use crate::{i18n, util::OptionValueOrElseExt};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;

include!("bridge/navigation.rs");
include!("bridge/ffi.rs");
include!("bridge/client.rs");
