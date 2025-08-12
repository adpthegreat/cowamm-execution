//! This script is used to vendor Truffle JSON artifacts to be used for code
//! generation with `ethcontract`. This is done instead of fetching contracts
//! at build time to reduce the risk of failure.

use {
    anyhow::Result,
    contracts::paths,
    ethcontract_generate::Source,
    serde_json::{Map, Value},
    std::{
        fs,
        path::{Path, PathBuf},
    },
    tracing_subscriber::EnvFilter,
};

fn main() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("LOG_FILTER").unwrap_or_else(|_| "warn,vendor=info".into()),
        )
        .init();

    if let Err(err) = run() {
        tracing::error!("Error vendoring contracts: {:?}", err);
        std::process::exit(-1);
    }
}

#[rustfmt::skip]
fn run() -> Result<()> {
    let vendor = Vendor::try_new()?;

    const ETHFLOW_VERSION: &str = "0.0.0-rc.3";

    vendor
        .full()
        .npm(
            "CowProtocolToken",
            "@cowprotocol/token@1.1.0/build/artifacts/src/contracts/CowProtocolToken.sol/\
             CowProtocolToken.json",
        )?
        .github(
            "CoWSwapEthFlow",
            &format!(
                "cowprotocol/ethflowcontract/{ETHFLOW_VERSION}-artifacts/hardhat-artifacts/src/\
                 CoWSwapEthFlow.sol/CoWSwapEthFlow.json"
            ),
        )?
        .npm(
            "ERC20Mintable",
            "@openzeppelin/contracts@2.5.0/build/contracts/ERC20Mintable.json",
        )?
        .npm(
            "GPv2AllowListAuthentication",
            // We use `_Implementation` because the use of a proxy contract makes
            // deploying  for the e2e tests more cumbersome.
            "@cowprotocol/contracts@1.1.2/deployments/mainnet/\
             GPv2AllowListAuthentication_Implementation.json",
        )?
        .npm(
            "GPv2Settlement",
            "@cowprotocol/contracts@1.1.2/deployments/mainnet/GPv2Settlement.json",
        )?
        .npm(
            "GnosisSafe",
            "@gnosis.pm/safe-contracts@1.3.0/build/artifacts/contracts/GnosisSafe.sol/GnosisSafe.\
             json",
        )?
        .npm(
            "GnosisSafeCompatibilityFallbackHandler",
            "@gnosis.pm/safe-contracts@1.3.0/build/artifacts/contracts/handler/\
             CompatibilityFallbackHandler.sol/CompatibilityFallbackHandler.json",
        )?
        .npm(
            "GnosisSafeProxy",
            "@gnosis.pm/safe-contracts@1.3.0/build/artifacts/contracts/\
             proxies/GnosisSafeProxy.sol/GnosisSafeProxy.json",
        )?
        .npm(
            "GnosisSafeProxyFactory",
            "@gnosis.pm/safe-contracts@1.3.0/build/artifacts/contracts/\
             proxies/GnosisSafeProxyFactory.sol/GnosisSafeProxyFactory.json",
        )?
        .manual(
            "HooksTrampoline",
            "Manually vendored ABI and bytecode for hooks trampoline contract",
        )
        .npm("WETH9", "canonical-weth@1.4.0/build/contracts/WETH9.json")?;

    vendor
        .abi_only()
        .github(
            "CoWSwapOnchainOrders",
            &format!(
                "cowprotocol/ethflowcontract/{ETHFLOW_VERSION}-artifacts/hardhat-artifacts/src/\
                 mixins/CoWSwapOnchainOrders.sol/CoWSwapOnchainOrders.json"
            ),
        )?
        .npm(
            "ERC20",
            "@openzeppelin/contracts@3.3.0/build/contracts/ERC20.json",
        )?
        .manual(
            "ERC1271SignatureValidator",
            "Manually vendored ABI for ERC-1271 signature validation",
        )
        .npm(
            "IUniswapLikeRouter",
            "@uniswap/v2-periphery@1.1.0-beta.0/build/IUniswapV2Router02.json",
        )?
        .manual(
            "ChainalysisOracle",
            "Chainalysis does not publish its code",
        );

    Ok(())
}

struct Vendor {
    artifacts: PathBuf,
}

impl Vendor {
    fn try_new() -> Result<Self> {
        let artifacts = paths::contract_artifacts_dir();
        tracing::info!("vendoring contract artifacts to '{}'", artifacts.display());
        fs::create_dir_all(&artifacts)?;
        Ok(Self { artifacts })
    }

    /// Creates a context for vendoring "full" contract data, including bytecode
    /// used for deploying the contract for end-to-end test.
    fn full(&self) -> VendorContext<'_> {
        VendorContext {
            artifacts: &self.artifacts,
            properties: &[
                ("abi", "abi,compilerOutput.abi"),
                ("devdoc", "devdoc,compilerOutput.devdoc"),
                ("userdoc", "userdoc"),
                ("bytecode", "bytecode"),
            ],
        }
    }

    /// Creates a context for vendoring only the contract ABI for generating
    /// bindings. This is preferred over [`Vendor::full`] for contracts that do
    /// not need to be deployed for tests, or get created by alternative means
    /// (e.g. `UniswapV2Pair` contracts don't require bytecode as they get
    /// created by `UniswapV2Factory` instances on-chain).
    fn abi_only(&self) -> VendorContext<'_> {
        VendorContext {
            artifacts: &self.artifacts,
            properties: &[
                ("abi", "abi,compilerOutput.abi"),
                ("devdoc", "devdoc,compilerOutput.devdoc"),
                ("userdoc", "userdoc"),
            ],
        }
    }
}

struct VendorContext<'a> {
    artifacts: &'a Path,
    properties: &'a [(&'a str, &'a str)],
}

impl VendorContext<'_> {
    fn npm(&self, name: &str, path: &str) -> Result<&Self> {
        self.vendor_source(name, Source::npm(path))
    }

    fn github(&self, name: &str, path: &str) -> Result<&Self> {
        self.vendor_source(
            name,
            Source::http(&format!("https://raw.githubusercontent.com/{path}"))?,
        )
    }

    fn manual(&self, name: &str, reason: &str) -> &Self {
        // We just keep these here to document that they are manually generated
        // and not pulled from some source.
        tracing::info!("skipping {}: {}", name, reason);
        self
    }

    fn retrieve_value_from_path<'a>(source: &'a Value, path: &'a str) -> Value {
        let mut current_value: &Value = source;
        for property in path.split('.') {
            current_value = &current_value[property];
        }
        current_value.clone()
    }

    fn vendor_source(&self, name: &str, source: Source) -> Result<&Self> {
        tracing::info!("retrieving {:?}", source);
        let artifact_json = source.artifact_json()?;

        tracing::debug!("pruning artifact JSON");
        let pruned_artifact_json = {
            let json = serde_json::from_str::<Value>(&artifact_json)?;
            let mut pruned = Map::new();
            for (property, paths) in self.properties {
                if let Some(value) = paths
                    .split(',')
                    .map(|path| Self::retrieve_value_from_path(&json, path))
                    .find(|value| !value.is_null())
                {
                    pruned.insert(property.to_string(), value);
                }
            }
            serde_json::to_string(&pruned)?
        };

        let path = self.artifacts.join(name).with_extension("json");
        tracing::debug!("saving artifact to {}", path.display());
        fs::write(path, pruned_artifact_json)?;

        Ok(self)
    }
}