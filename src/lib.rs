use std::fmt::Debug;

use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, RootProvider,
    },
    transports::http::{Client, Http},
};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{error::OctaneError, messenger::Messager};

pub mod agent;
pub mod error;
pub mod machine;
pub mod messenger;
pub mod world;

// pub type AnvilProvider = FillProvider<JoinFill<Identity, WalletFiller<EthereumWallet>>;
pub type AnvilProvider = FillProvider<
    JoinFill<
        JoinFill<
            JoinFill<
                JoinFill<alloy::providers::Identity, GasFiller>, // First JoinFill
                NonceFiller,                                     // Second JoinFill's right argument
            >,
            ChainIdFiller, // Third JoinFill's right argument
        >,
        WalletFiller<EthereumWallet>, // Fourth JoinFill's right argument, assuming you want to add this
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;
