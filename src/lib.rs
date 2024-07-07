use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use tracing::trace;
use alloy::{network::EthereumWallet};
use alloy::transports::http::Client;
use alloy::providers::RootProvider;
use alloy::providers::Identity;
use alloy::providers::fillers::FillProvider;
use alloy::providers::fillers::JoinFill;
use alloy::providers::fillers::WalletFiller;
use alloy::network::Ethereum;
use alloy::transports::http::Http;
use crate::{error::OctaneError, messenger::Messager};

pub mod error;
pub mod machine;
pub mod messenger;
pub mod agent;
pub mod world;

pub type AnvilProvider = FillProvider<JoinFill<Identity, WalletFiller<EthereumWallet>>, RootProvider<Http<Client>>, Http<Client>, Ethereum>;