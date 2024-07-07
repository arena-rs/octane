use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{error::ArbiterEngineError, messenger::Messager};

pub mod error;
pub mod machine;
pub mod messenger;
