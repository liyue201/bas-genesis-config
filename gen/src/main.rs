use ethereum_types::{H160, H256, U256};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;
use anyhow::{Result, anyhow};
use ethabi::{Function, Param, ParamType, Contract, Token};

static STAKING_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0x0000000000000000000000000000000000001000").unwrap());
static SLASHING_INDICATOR_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0x0000000000000000000000000000000000001001").unwrap());
static SYSTEM_REWORD_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0x0000000000000000000000000000000000001002").unwrap());
static STAKING_POOL_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0x0000000000000000000000000000000000007001").unwrap());
static GOVERNANCE_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0x0000000000000000000000000000000000007002").unwrap());
static CHAIN_CONFIG_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0x0000000000000000000000000000000000007003").unwrap());
static RUNTIME_UPGRADE_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0x0000000000000000000000000000000000007004").unwrap());
static DEPLOYER_PROXY_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0x0000000000000000000000000000000000007005").unwrap());
static INTERMEDIARY_SYSTEM_ADDRESS: Lazy<H160> =
    Lazy::new(|| H160::from_str("0xfffffffffffffffffffffffffffffffffffffffe").unwrap());

#[derive(Debug, Serialize, Deserialize)]
struct ArtifactData {
    #[serde(alias = "bytecode")]
    byte_code: String,
    #[serde(alias = "deployedBytecode")]
    deployed_byte_code: String,
}

#[derive(RustEmbed)]
#[folder = "build/"]
struct Asset;

impl Asset {
    fn staking_artifact() -> ArtifactData {
        Self::artifact("contracts/Staking.json")
    }
    fn artifact(filename: &str) -> ArtifactData {
        let data: std::borrow::Cow<'static, [u8]> = Asset::get(filename).unwrap();
        serde_json::from_slice(data.as_ref().into()).unwrap()
    }

    fn staking_contract() -> Contract {
        Self::contract("abi/Staking.json")
    }

    fn contract(filename: &str) -> Contract {
        let data: std::borrow::Cow<'static, [u8]> = Asset::get(filename).unwrap();
        let data: &[u8] = data.as_ref();

        //let str: String = String::from_utf8(data.to_vec()).unwrap();
        //println!("str = {}", str);

        ethabi::Contract::load(data).unwrap()
    }
}

// #[derive(RustEmbed)]
// #[folder = "build/abi"]
// struct ABI;
//
// impl ABI {
//     fn staking_contract() -> Contract {
//         Self::contract("Strings.json")
//     }
//
//     fn contract(filename: &str) -> Contract {
//         let data: std::borrow::Cow<'static, [u8]> = Asset::get(filename).unwrap();
//
//         let data: &[u8] = data.as_ref();
//
//         let str: String = String::from_utf8(data.to_vec()).unwrap();
//         println!("str = {}", str);
//
//         ethabi::Contract::load(data).unwrap()
//     }
// }


#[derive(Serialize, Deserialize)]
struct ParliaConfig {
    period: u64,
    epoch: u64,
}

#[derive(Serialize, Deserialize)]
struct ChainConfig {
    chain_id: i64,

    homestead_block: Option<U256>,

    eip150_block: Option<U256>,
    eip150_hash: Option<U256>,
    eip155_block: Option<U256>,
    eip158_block: Option<U256>,

    byzantium_block: Option<U256>,
    constantinople_block: Option<U256>,
    petersburg_block: Option<U256>,
    istanbul_block: Option<U256>,
    muir_glacier_block: Option<U256>,
    berlin_block: Option<U256>,
    runtime_upgrade_block: Option<U256>,
    deployer_proxy_block: Option<U256>,

    yolo_v3_block: Option<U256>,

    ewasm_block: Option<U256>,
    catalyst_block: Option<U256>,

    ramanujan_block: Option<U256>,
    niels_block: Option<U256>,

    mirror_sync_block: Option<U256>,

    bruno_block: Option<U256>,

    // Various consensus engines
    //Clique * CliqueConfig `json:"clique,omitempty" toml:",omitempty"`
    parlia: Option<ParliaConfig>,
}

#[derive(Serialize, Deserialize)]
struct GenesisAccount {
    code: Vec<u8>,
    storage: HashMap<H160, H256>,
    balance: Option<H256>,
    nonce: u64,
    privateKey: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Genesis {
    config: ChainConfig,
    nonce: u64,
    timestamp: u64,
    extra_data: Vec<u8>,
    gas_limit: u64,
    difficulty: U256,
    mix_hash: U256,
    coinbase: H160,
    alloc: HashMap<H160, GenesisAccount>,
    number: u64,
    gas_used: u64,
    parent_hash: H256,
}

impl Genesis {
    pub fn default() -> Self {
        Genesis {
            config: ChainConfig {
                chain_id: Default::default(),
                homestead_block: None,
                eip150_block: None,
                eip150_hash: None,
                eip155_block: None,
                eip158_block: None,
                byzantium_block: None,
                constantinople_block: None,
                petersburg_block: None,
                istanbul_block: None,
                muir_glacier_block: None,
                berlin_block: None,
                runtime_upgrade_block: None,
                deployer_proxy_block: None,
                yolo_v3_block: None,
                ewasm_block: None,
                catalyst_block: None,
                ramanujan_block: None,
                niels_block: None,
                mirror_sync_block: None,
                bruno_block: None,
                parlia: Some(ParliaConfig {
                    period: 3,
                    epoch: 0,
                }),
            },
            nonce: 0,
            timestamp: 0x5e9da7ce,
            extra_data: vec![],
            gas_limit: 0x2625a00,
            difficulty: 1u32.into(),
            mix_hash: Default::default(),
            coinbase: Default::default(),
            alloc: Default::default(),
            number: 0,
            gas_used: 0,
            parent_hash: Default::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct ConsensusParams {
    #[serde(alias = "activeValidatorsLength")]
    active_validators_length: u32,
    #[serde(alias = "epochBlockInterval")]
    epoch_block_interval: u64,
    #[serde(alias = "misdemeanorThreshold")]
    misdemeanor_threshold: u32,
    #[serde(alias = "felonyThreshold")]
    felony_threshold: u32,
    #[serde(alias = "validatorJailEpochLength")]
    validator_jail_epoch_length: u32,
    #[serde(alias = "undelegatePeriod")]
    undelegate_period: u32,
    #[serde(alias = "minValidatorStakeAmount")]
    min_validator_stake_amount: U256,
    #[serde(alias = "minStakingAmount")]
    min_staking_amount: U256,
}

#[derive(Clone, Serialize, Deserialize)]
struct GenesisConfig {
    #[serde(alias = "chainId")]
    chain_id: i64,
    deployers: Vec<H160>,
    validators: Vec<H160>,
    #[serde(alias = "systemTreasury")]
    system_treasury: Option<H160>,
    #[serde(alias = "consensusParams")]
    consensus_params: ConsensusParams,
    #[serde(alias = "votingPeriod")]
    voting_period: i64,
    faucet: HashMap<H160, String>,
    #[serde(alias = "commissionRate")]
    commission_rate: i64,
    #[serde(alias = "initialStakes")]
    initial_stakes: HashMap<H160, U256>,
}

static DEV_NET: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    chain_id: 14000,
    deployers: vec![],
    validators: vec![
        H160::from_str("0x08fae3885e299c24ff9841478eb946f41023ac69").unwrap(),
        H160::from_str("0x751aaca849b09a3e347bbfe125cf18423cc24b40").unwrap(),
        H160::from_str("0xa6ff33e3250cc765052ac9d7f7dfebda183c4b9b").unwrap(),
        H160::from_str("0x49c0f7c8c11a4c80dc6449efe1010bb166818da8").unwrap(),
        H160::from_str("0x8e1ea6eaa09c3b40f4a51fcd056a031870a0549a").unwrap(),
    ],
    system_treasury: None,
    consensus_params: ConsensusParams {
        active_validators_length: 25,
        epoch_block_interval: 12000,
        misdemeanor_threshold: 50,
        felony_threshold: 150,
        validator_jail_epoch_length: 7,
        undelegate_period: 6,
        min_validator_stake_amount: U256::from_str("0xde0b6b3a7640000").unwrap(),
        min_staking_amount: U256::from_str("0xde0b6b3a7640000").unwrap(),
    },
    voting_period: 60,
    faucet: HashMap::from([
        (
            H160::from_str("0x00a601f45688dba8a070722073b015277cf36725").unwrap(),
            "0x21e19e0c9bab2400000".into(),
        ),
        (
            H160::from_str("0xb891fe7b38f857f53a7b5529204c58d5c487280b").unwrap(),
            "0x52b7d2dcc80cd2e4000000".into(),
        ),
    ]),
    commission_rate: 0,
    initial_stakes: HashMap::from([
        (
            H160::from_str("0x08fae3885e299c24ff9841478eb946f41023ac69").unwrap(),
            "0x3635c9adc5dea00000".into(),
        ),
        (
            H160::from_str("0x751aaca849b09a3e347bbfe125cf18423cc24b40").unwrap(),
            "0x3635c9adc5dea00000".into(),
        ),
        (
            H160::from_str("0xa6ff33e3250cc765052ac9d7f7dfebda183c4b9b").unwrap(),
            "0x3635c9adc5dea00000".into(),
        ),
        (
            H160::from_str("0x49c0f7c8c11a4c80dc6449efe1010bb166818da8").unwrap(),
            "0x3635c9adc5dea00000".into(),
        ),
        (
            H160::from_str("0x8e1ea6eaa09c3b40f4a51fcd056a031870a0549a").unwrap(),
            "0x3635c9adc5dea00000".into(),
        ),
    ]),
});


fn create_extra_data(validators: Vec<H160>) -> Vec<u8> {
    //todo:
    return vec![];
}

fn invoke_constructor(genesis: Genesis, contract_address: H160, artifact: ArtifactData, contract: Contract, inputs: &[Token]) -> Result<()> {
    //println!("artifact: {:?}", artifact);

    let ctor = contract.functions.get("ctor").unwrap();
    let sig = ctor[0].short_signature();

    println!("sig: {:?}", sig);

    ctor[0].encode_input(inputs)?;
    Ok(())
}

fn validators_to_tokens(validators: Vec<H160>) -> Vec<Token> {
    let mut tokens = vec![];
    for &v in &validators {
        tokens.push(Token::Address(v));
    }
    tokens
}

fn create_genesis_config(cfg: GenesisConfig, filename: &str) -> Result<()> {
    let mut genesis = Genesis::default();
    genesis.config.chain_id = cfg.chain_id;
    genesis.extra_data = create_extra_data(cfg.validators.clone());
    genesis.config.parlia = Some(ParliaConfig { epoch: cfg.consensus_params.epoch_block_interval, period: 3 });

    let mut initial_stakes = vec![];
    let mut initial_stake_total: U256 = 0u32.into();

    for &v in &cfg.validators {
        let stake = cfg.initial_stakes.get(&v);
        if stake.is_some() {
            let stake = stake.unwrap();
            initial_stakes.push(stake);
            initial_stake_total = initial_stake_total.add(stake);
        }
    }
    println!("initial_stakes = {:?}", initial_stakes);
    println!("initial_stake_total = {:?}", initial_stake_total);
    // invokeConstructorOrPanic(genesis, stakingAddress, stakingRawArtifact, []string{"address[]", "uint256[]", "uint16"}, []interface{}{
    //     config.Validators,
    //     initialStakes,
    //     uint16(config.CommissionRate),
    // }, silent)

    invoke_constructor(genesis, STAKING_ADDRESS.clone(), Asset::staking_artifact(),
                       Asset::staking_contract(), validators_to_tokens(cfg.validators).as_slice());

    //Token::Uint(uint.into()
    Ok(())
}

fn main() {
    //H160::from_str("0x0000000000000000000000000000000000001000");
    //let index_html = Asset::get("Staking.json").unwrap();
    //println!("{:?}", std::str::from_utf8(index_html.as_ref()));
    create_genesis_config(DEV_NET.clone(), "devnet.json");
    println!("Hello, world!");
}
