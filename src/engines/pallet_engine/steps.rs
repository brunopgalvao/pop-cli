use super::{pallet_entry::AddPalletEntry, PalletEngine};
use crate::commands::add::AddPallet;
use anyhow::{bail, Result};
use dependency::Dependency;
use log::error;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use Steps::*;
/// Define the steps needed for a particular pallet insertion
pub(super) enum Steps {
    /// Import statements for pallet
    RuntimePalletImport(TokenStream2),
    /// Every pallet must impl pallet::Config for Runtime
    RuntimePalletConfiguration(TokenStream2),
    /// The runtime/Cargo.toml needs an import for the pallet being inserted
    /// This includes features [try-runtime, runtime-benchmarks, std], path information for `git` or local path
    RuntimePalletDependency(Dependency),
    /// ConstructRuntime! entry
    ConstructRuntimeEntry(AddPalletEntry),
    /// Include a `list_benchmarks!` entry
    ListBenchmarks(String),
    /// Does pallet require a genesis configuration?
    ChainspecGenesisConfig(String),
    /// ChainSpec imports if necessary
    ChainspecGenesisImport(TokenStream2),
    /// Node specific imports if the above two are required
    NodePalletDependency(Dependency),
}

pub(super) fn step_builder(pallet: AddPallet) -> Result<Vec<Steps>> {
    let mut steps: Vec<Steps> = vec![];
    match pallet {
        // Adding a pallet-parachain-template requires 5 distinct steps
        AddPallet::Template => {
            steps.push(RuntimePalletDependency(Dependency::runtime_template()));
            steps.push(RuntimePalletImport(quote!(
                pub use pallet_parachain_template;
            )));
            steps.push(RuntimePalletConfiguration(quote!(
                /// Configure the pallet template in pallets/template.
                impl pallet_parachain_template::Config for Runtime {
                    type RuntimeEvent = RuntimeEvent;
                }
            )));
            steps.push(ConstructRuntimeEntry(AddPalletEntry::new(
                // Index
                None,
                // Path
                "pallet_parachain_template",
                // Pallet name
                // TODO (high priority): implement name conflict resolution strategy
                "Template",
            )));
            steps.push(NodePalletDependency(Dependency::node_template()))
        }
        AddPallet::Frame(_) => unimplemented!("Frame pallets not yet implemented"),
    };
    Ok(steps)
}

pub(super) fn run_steps(mut pe: PalletEngine, steps: Vec<Steps>) -> Result<()> {
    use super::State::*;
    pe.prepare_output()?;
    for step in steps.into_iter() {
        match step {
            RuntimePalletImport(stmt) => {
                match pe.state {
                    Init => {
                        warn!("Non fatal: `prepare_output` was not called");
                        pe.state = Import;
                        pe.insert_import(stmt);
                    }
                    Import => pe.insert_import(stmt),
                    _ => {
                        // We don't support writing import statements in any other engine state 
                        // Log non-fatal error and continue
                        error!("Cannot write import stmts. Check step builder");
                        continue;
                    }
                }
                pe.insert_import(quote!(
                    pub use pallet_parachain_template;
                ));
            }
            // RuntimePalletConfiguration(step) => pe.insert(step),
            // RuntimePalletDependency(step) => pe.insert(step),
            // ConstructRuntimeEntry(step) => pe.insert(step),
            // ListBenchmarks(step) => pe.insert(step),
            // ChainspecGenesisConfig(step) => pe.insert(step),
            // ChainspecGenesisImport(step) => pe.insert(step),
            // NodePalletDependency(step) => pe.insert(step),
            _ => {
                unimplemented!()
            }
        }
    }
    Ok(())
}

mod dependency {
    use strum_macros::{Display, EnumString};

    #[derive(EnumString, Display)]
    pub(super) enum Features {
        #[strum(serialize = "std")]
        Std,
        #[strum(serialize = "runtime-benchmarks")]
        RuntimeBenchmarks,
        #[strum(serialize = "try-runtime")]
        TryRuntime,
        Custom(String),
    }
    pub(super) struct Dependency {
        features: Vec<Features>,
        path: String,
        no_default_features: bool,
    }

    impl Dependency {
        /// Dependencies required for adding a pallet-parachain-template to runtime
        pub(super) fn runtime_template() -> Self {
            Self {
                features: vec![
                    Features::RuntimeBenchmarks,
                    Features::TryRuntime,
                    Features::Std,
                ],
                // TODO hardcode for now
                path: format!(r#"path = "../pallets/template""#),
                no_default_features: true,
            }
        }
        /// Dependencies required for adding a pallet-parachain-template to node
        pub(super) fn node_template() -> Self {
            Self {
                features: vec![Features::RuntimeBenchmarks, Features::TryRuntime],
                // TODO hardcode for now
                path: format!(r#"path = "../pallets/template""#),
                no_default_features: false,
            }
        }
    }
}