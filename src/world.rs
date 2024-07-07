//! The world module contains the core world abstraction for the Arbiter Engine.

use std::collections::VecDeque;

use futures_util::future::join_all;
use alloy::node_bindings::{Anvil, AnvilInstance};
use alloy::providers::ProviderBuilder;
use std::sync::Arc;
use std::collections::HashMap;
use alloy::signers::local::PrivateKeySigner;

use tokio::spawn;

use super::*;
use crate::{
    agent::{Agent, AgentBuilder},
    machine::MachineInstruction,
};

/// A world is a collection of agents that use the same type of provider, e.g.,
/// operate on the same blockchain or same `Environment`. The world is
/// responsible for managing the agents and their state transitions.
///
/// # How it works
/// The [`World`] holds on to a collection of [`Agent`]s and can run them all
/// concurrently when the [`run`] method is called. The [`World`] takes in
/// [`AgentBuilder`]s and when it does so, it creates [`Agent`]s that are now
/// connected to the world via a client ([`Arc<RevmMiddleware>`]) and a messager
/// ([`Messager`]).
pub struct World {
    /// The identifier of the world.
    pub id: String,

    /// The agents in the world.
    pub agents: Option<HashMap<String, Agent>>,

    /// The environment for the world.
    pub environment: AnvilInstance,

    /// The messaging layer for the world.
    pub messager: Messager,
}

impl World {
    /// Creates a new [`World`] with the given identifier and provider.
    pub fn new(id: &str) -> Self {
        let port = 8545u16;

        let anvil = Anvil::new()
            .port(port)
            .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
            .spawn();

        Self {
            id: id.to_owned(),
            agents: Some(HashMap::new()),
            environment: anvil,
            messager: Messager::new(),
        }
    }

    /// Adds an agent, constructed from the provided `AgentBuilder`, to the
    /// world.
    ///
    /// This method takes an `AgentBuilder` instance, extracts its identifier,
    /// and uses it to create both a `RevmMiddleware` client and a
    /// `Messager` specific to the agent. It then builds the `Agent` from
    /// the `AgentBuilder` using these components. Finally, the newly
    /// created `Agent` is inserted into the world's internal collection of
    /// agents.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// - It fails to create a `RevmMiddleware` client for the agent.
    /// - The `AgentBuilder` fails to build the `Agent`.
    /// - The world's internal collection of agents is not initialized.
    ///
    /// # Examples
    ///
    /// Assuming you have an `AgentBuilder` instance named `agent_builder`:
    ///
    /// ```ignore
    /// world.add_agent(agent_builder);
    /// ```
    ///
    /// This will add the agent defined by `agent_builder` to the world.
    pub fn add_agent(&mut self, agent_builder: AgentBuilder) {
        let id = agent_builder.id.clone();

        // Set up signer from the first default Anvil account (Alice).
        let signer: PrivateKeySigner = self.environment.keys()[0].clone().into();
        let wallet = EthereumWallet::from(signer);

        // Create a provider with the wallet.
        let rpc_url = self.environment.endpoint().parse().unwrap();
        let client =
            ProviderBuilder::new().wallet(wallet).on_http(rpc_url);

        let messager = self.messager.for_agent(&id);

        let agent = agent_builder
            .build(Arc::new(client), messager)
            .expect("Failed to build agent from AgentBuilder");

        let agents = self
            .agents
            .as_mut()
            .expect("Agents collection not initialized");

        agents.insert(id.to_owned(), agent);
    }

    /// Executes all agents and their behaviors concurrently within the world.
    ///
    /// This method takes all the agents registered in the world and runs their
    /// associated behaviors in parallel. Each agent's behaviors are
    /// executed with their respective messaging and client context. This
    /// method ensures that all agents and their behaviors are started
    /// simultaneously, leveraging asynchronous execution to manage concurrent
    /// operations.
    ///
    /// # Errors
    ///
    /// Returns an error if no agents are found in the world, possibly
    /// indicating that the world has already been run or that no agents
    /// were added prior to execution.
    pub async fn run(&mut self) -> Result<&AnvilInstance, ArbiterEngineError> {
        let agents = match self.agents.take() {
            Some(agents) => agents,
            None => {
                return Err(ArbiterEngineError::WorldError(
                    "No agents found. Has the world already been ran?".to_owned(),
                ))
            }
        };
        let mut tasks = vec![];
        // Prepare a queue for messagers corresponding to each behavior engine.
        let mut messagers = VecDeque::new();
        // Populate the messagers queue.
        for (_, agent) in agents.iter() {
            for _ in &agent.behavior_engines {
                messagers.push_back(agent.messager.clone());
            }
        }
        // For each agent, spawn a task for each of its behavior engines.
        // Unwrap here is safe as we just built the dang thing.
        for (_, mut agent) in agents {
            for mut engine in agent.behavior_engines.drain(..) {
                let client = agent.client.clone();
                let messager = messagers.pop_front().unwrap();
                tasks.push(spawn(async move {
                    engine
                        .execute(MachineInstruction::Start(client, messager))
                        .await
                }));
            }
        }
        // Await the completion of all tasks.
        join_all(tasks).await;

        Ok(&self.environment)
    }
}