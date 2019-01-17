//! Service and ServiceFactory implementation. Specialized wrapper over Substrate service.

#![warn(unused_extern_crates)]

use basic_authorship::ProposerFactory;
use client;
use consensus::{import_queue, start_aura, AuraImportQueue, NothingExtra, SlotDuration};
use node_executor;
use primitives::ed25519::Pair;
use runtime_primitives::BasicInherentData as InherentData;
use rust_substrate_prototype_runtime::{self, opaque::Block, GenesisConfig, RuntimeApi};
use std::sync::Arc;
use substrate_service::{
    FactoryFullConfiguration, FullBackend, FullClient, FullComponents, FullExecutor, LightBackend,
    LightClient, LightComponents, LightExecutor, TaskExecutor,
};
use transaction_pool::{self, txpool::Pool as TransactionPool};

pub use substrate_executor::NativeExecutor;
// Our native executor instance.
native_executor_instance!(
	pub Executor,
	rust_substrate_prototype_runtime::api::dispatch,
	rust_substrate_prototype_runtime::native_version,
	include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/rust_substrate_prototype_runtime.compact.wasm")
);

construct_simple_protocol! {
    /// Demo protocol attachment for substrate.
    pub struct NodeProtocol where Block = Block { }
}

construct_service_factory! {
    struct Factory {
        Block = Block,
        RuntimeApi = RuntimeApi,
        NetworkProtocol = NodeProtocol { |config| Ok(NodeProtocol::new()) },
        RuntimeDispatch = node_executor::Executor,
        FullTransactionPoolApi = transaction_pool::ChainApi<client::Client<FullBackend<Self>, FullExecutor<Self>, Block, RuntimeApi>, Block>
            { |config, client| Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client))) },
        LightTransactionPoolApi = transaction_pool::ChainApi<client::Client<LightBackend<Self>, LightExecutor<Self>, Block, RuntimeApi>, Block>
            { |config, client| Ok(TransactionPool::new(config, transaction_pool::ChainApi::new(client))) },
        Genesis = GenesisConfig,
        Configuration = (),
        FullService = FullComponents<Self>
            { |config: FactoryFullConfiguration<Self>, executor: TaskExecutor|
                FullComponents::<Factory>::new(config, executor)
            },
        AuthoritySetup = {
            |service: Self::FullService, executor: TaskExecutor, key: Option<Arc<Pair>>| {
                if let Some(key) = key {
                    info!("Using authority key {}", key.public());
                    let proposer = Arc::new(ProposerFactory {
                        client: service.client(),
                        transaction_pool: service.transaction_pool(),
                    });
                    let client = service.client();
                    executor.spawn(start_aura(
                        SlotDuration::get_or_compute(&*client)?,
                        key.clone(),
                        client.clone(),
                        client,
                        proposer,
                        service.network(),
                        service.on_exit(),
                    ));
                }

                Ok(service)
            }
        },
        LightService = LightComponents<Self>
            { |config, executor| <LightComponents<Factory>>::new(config, executor) },
        FullImportQueue = AuraImportQueue<
            Self::Block,
            FullClient<Self>,
            NothingExtra,
            ::consensus::InherentProducingFn<InherentData>,
        >
            { |config: &mut FactoryFullConfiguration<Self> , client: Arc<FullClient<Self>>|
                Ok(import_queue(
                    SlotDuration::get_or_compute(&*client)?,
                    client,
                    NothingExtra,
                    ::consensus::make_basic_inherent as _,
                ))
            },
        LightImportQueue = AuraImportQueue<
            Self::Block,
            LightClient<Self>,
            NothingExtra,
            ::consensus::InherentProducingFn<InherentData>,
        >
            { |ref mut config, client: Arc<LightClient<Self>>|
                Ok(import_queue(
                    SlotDuration::get_or_compute(&*client)?,
                    client,
                    NothingExtra,
                    ::consensus::make_basic_inherent as _,
                ))
            },
    }
}
