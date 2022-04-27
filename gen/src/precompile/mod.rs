use std::collections::BTreeMap;
use ethjson::spec::ForkSpec;
use evm::executor::stack::{PrecompileFailure, PrecompileFn, PrecompileOutput};
use evm::{ExitError, ExitSucceed};
use web3::Transport;
use evm::backend::{ApplyBackend, MemoryAccount, MemoryBackend, MemoryVicinity};
use lazy_static::lazy_static;
use libsecp256k1::SecretKey;
use sha3::{Digest, Keccak256};
use primitive_types::{H160, H256, U256};
use serde::Deserialize;
use std::convert::TryInto;
use std::str::FromStr;
use evm::executor::stack::{
    MemoryStackState, StackExecutor,
    StackSubstateMetadata,
};
use evm::{Config, Context};


lazy_static! {
	static ref ISTANBUL_BUILTINS: BTreeMap<H160, ethcore_builtin::Builtin> =
		JsonPrecompile::builtins("./res/istanbul_builtins.json");
}

lazy_static! {
	static ref BERLIN_BUILTINS: BTreeMap<H160, ethcore_builtin::Builtin> =
		JsonPrecompile::builtins("./res/berlin_builtins.json");
}

macro_rules! precompile_entry {
	($map:expr, $builtins:expr, $index:expr) => {
		let x: fn(
			&[u8],
			Option<u64>,
			&Context,
			bool,
		) -> Result<PrecompileOutput, PrecompileFailure> =
			|input: &[u8], gas_limit: Option<u64>, _context: &Context, _is_static: bool| {
				let builtin = $builtins.get(&H160::from_low_u64_be($index)).unwrap();
				Self::exec_as_precompile(builtin, input, gas_limit)
			};
		$map.insert(H160::from_low_u64_be($index), x);
	};
}

pub struct JsonPrecompile;

impl JsonPrecompile {
    pub fn precompile(spec: &ForkSpec) -> Option<BTreeMap<H160, PrecompileFn>> {
        match spec {
            ForkSpec::Istanbul => {
                let mut map = BTreeMap::new();
                precompile_entry!(map, ISTANBUL_BUILTINS, 1);
                precompile_entry!(map, ISTANBUL_BUILTINS, 2);
                precompile_entry!(map, ISTANBUL_BUILTINS, 3);
                precompile_entry!(map, ISTANBUL_BUILTINS, 4);
                precompile_entry!(map, ISTANBUL_BUILTINS, 5);
                precompile_entry!(map, ISTANBUL_BUILTINS, 6);
                precompile_entry!(map, ISTANBUL_BUILTINS, 7);
                precompile_entry!(map, ISTANBUL_BUILTINS, 8);
                precompile_entry!(map, ISTANBUL_BUILTINS, 9);
                Some(map)
            }
            ForkSpec::Berlin => {
                let mut map = BTreeMap::new();
                precompile_entry!(map, BERLIN_BUILTINS, 1);
                precompile_entry!(map, BERLIN_BUILTINS, 2);
                precompile_entry!(map, BERLIN_BUILTINS, 3);
                precompile_entry!(map, BERLIN_BUILTINS, 4);
                precompile_entry!(map, BERLIN_BUILTINS, 5);
                precompile_entry!(map, BERLIN_BUILTINS, 6);
                precompile_entry!(map, BERLIN_BUILTINS, 7);
                precompile_entry!(map, BERLIN_BUILTINS, 8);
                precompile_entry!(map, BERLIN_BUILTINS, 9);
                Some(map)
            }
            // precompiles for London and Berlin are the same
            ForkSpec::London => Self::precompile(&ForkSpec::Berlin),
            _ => None,
        }
    }

    fn builtins(spec_path: &str) -> BTreeMap<H160, ethcore_builtin::Builtin> {
        let reader = std::fs::File::open(spec_path).unwrap();
        let builtins: BTreeMap<ethjson::hash::Address, ethjson::spec::builtin::BuiltinCompat> =
            serde_json::from_reader(reader).unwrap();
        builtins
            .into_iter()
            .map(|(address, builtin)| {
                (
                    address.into(),
                    ethjson::spec::Builtin::from(builtin).try_into().unwrap(),
                )
            })
            .collect()
    }

    fn exec_as_precompile(
        builtin: &ethcore_builtin::Builtin,
        input: &[u8],
        gas_limit: Option<u64>,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
        let cost = builtin.cost(input, 0);

        if let Some(target_gas) = gas_limit {
            if cost > U256::from(u64::MAX) || target_gas < cost.as_u64() {
                return Err(PrecompileFailure::Error {
                    exit_status: ExitError::OutOfGas,
                });
            }
        }

        let mut output = Vec::new();
        match builtin.execute(input, &mut parity_bytes::BytesRef::Flexible(&mut output)) {
            Ok(()) => Ok(PrecompileOutput {
                exit_status: ExitSucceed::Stopped,
                output,
                cost: cost.as_u64(),
                logs: Vec::new(),
            }),
            Err(e) => Err(PrecompileFailure::Error {
                exit_status: ExitError::Other(e.into()),
            }),
        }
    }
}