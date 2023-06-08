use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    to_vec, Addr, Binary, Coin, ContractResult, CosmosMsg, CustomMsg, CustomQuery, Decimal256,
    Deps, QueryRequest, StdError, StdResult, SystemResult,
};

/// Instantiate Message
#[cw_serde]
pub struct InstantiateMsg {}

/// Execute Messages
#[cw_serde]
pub enum ExecuteMsg {
    Delegate {
        validator_address: Addr,
        amount: Coin,
    },
    Undelegate {
        validator_address: Addr,
        amount: Coin,
    },
    Redelegate {
        validator_src_address: Addr,
        validator_dst_address: Addr,
        amount: Coin,
    },
    ClaimDelegationRewards {
        validator_address: Addr,
        denom: String,
    },
}

/// Alliance Messages
#[cw_serde]
pub enum AllianceMsg {
    Delegate {
        delegator_address: Addr,
        validator_address: Addr,
        amount: Coin,
    },
    Undelegate {
        delegator_address: Addr,
        validator_address: Addr,
        amount: Coin,
    },
    Redelegate {
        delegator_address: Addr,
        validator_src_address: Addr,
        validator_dst_address: Addr,
        amount: Coin,
    },
    ClaimDelegationRewards {
        delegator_address: Addr,
        validator_address: Addr,
        denom: String,
    },
}

/// Chain message wrapper
#[cw_serde]
pub enum NoriaMsg {
    Alliance(AllianceMsg),
}

/// Chain query wrapper
#[cw_serde]
#[derive(QueryResponses)]
pub enum NoriaQuery {
    /// Alliance-specific queries, must be wrapped in 'alliance'
    #[returns(AllianceQuery)]
    Alliance(AllianceQuery),
}

#[cw_serde]
pub struct Pagination {
    pub key: Option<Binary>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub count_total: Option<bool>,
    pub reverse: Option<bool>,
}

#[cw_serde]
pub struct PaginationResponse {
    pub next_key: Option<Binary>,
    pub total: Option<u64>,
}

/// Alliance-specific queries
#[cw_serde]
#[derive(QueryResponses)]
pub enum AllianceQuery {
    #[returns(AllianceAllianceResponse)]
    Alliance { denom: String },

    #[returns(AllianceAlliancesResponse)]
    Alliances { pagination: Option<Pagination> },

    #[returns(AllianceAlliancesDelegationsResponse)]
    AlliancesDelegations { pagination: Option<Pagination> },

    #[returns(AllianceAlliancesDelegationsResponse)]
    AlliancesDelegationByValidator {
        delegator_addr: Addr,
        validator_addr: Addr,
        pagination: Option<Pagination>,
    },

    #[returns(SingleDelegationResponse)]
    Delegation {
        delegator_addr: Addr,
        validator_addr: Addr,
        denom: String,
    },

    #[returns(RewardsResponse)]
    DelegationRewards {
        delegator_addr: Addr,
        validator_addr: Addr,
        denom: String,
    },

    #[returns(AllianceParamsResponse)]
    Params {},

    #[returns(ValidatorResponse)]
    Validator { validator_addr: Addr },

    #[returns(AllValidatorsResponse)]
    Validators { pagination: Option<Pagination> },
}

#[cw_serde]
pub struct AllianceParams {
    pub reward_delay_time: u64,
    pub take_rate_claim_interval: u64,
    pub last_take_rate_claim_time: String,
}

#[cw_serde]
pub struct DecCoin {
    pub denom: Option<String>,
    pub amount: Decimal256,
}

#[cw_serde]
pub struct ValidatorResponse {
    pub validator_addr: Addr,
    pub total_delegation_shares: Vec<DecCoin>,
    pub validator_shares: Vec<DecCoin>,
    pub total_staked: Vec<DecCoin>,
}

#[cw_serde]
pub struct AllValidatorsResponse {
    pub validators: Vec<ValidatorResponse>,
    pub pagination: Option<PaginationResponse>,
}

#[cw_serde]
pub struct AllianceParamsResponse {
    pub params: AllianceParams,
}

#[cw_serde]
pub struct WeightRange {
    pub min: Decimal256,
    pub max: Decimal256,
}

#[cw_serde]
pub struct AllianceAsset {
    pub denom: String,
    pub reward_weight: Decimal256,
    pub consensus_weight: Decimal256,
    pub consensus_cap: Decimal256,
    pub take_rate: Decimal256,
    pub total_tokens: Decimal256,
    pub total_validator_shares: Decimal256,
    pub reward_start_time: String, // is there a better way to serde a golang time.Time string ("2023-06-06T18:37:29.956787974Z") ?
    pub reward_change_rate: Decimal256,
    pub reward_change_interval: u64,
    pub last_reward_change_time: String,
    pub reward_weight_range: WeightRange,
    pub is_initialized: Option<bool>,
}

#[cw_serde]
pub struct AllianceAllianceResponse {
    pub alliance: AllianceAsset,
}

#[cw_serde]
pub struct AllianceAlliancesResponse {
    pub alliances: Vec<AllianceAsset>,
    pub pagination: Option<PaginationResponse>,
}

#[cw_serde]
pub struct AllianceAlliancesDelegationsResponse {
    pub delegations: Option<Vec<DelegationResponse>>,
    pub pagination: Option<PaginationResponse>,
}

#[cw_serde]
pub struct RewardsResponse {
    pub rewards: Vec<Coin>,
}

#[cw_serde]
pub struct SingleDelegationResponse {
    pub delegation: DelegationResponse,
}

#[cw_serde]
pub struct DelegationResponse {
    pub delegation: Delegation,
    pub balance: Coin,
}

#[cw_serde]
pub struct Delegation {
    pub delegator_address: Option<Addr>,
    pub validator_address: Option<Addr>,
    pub denom: Option<String>,
    pub shares: Decimal256,
    pub reward_history: Option<Vec<Option<Reward>>>,
    pub last_reward_claim_height: Option<u64>,
}

#[cw_serde]
pub struct Reward {
    pub denom: Option<String>,
    pub index: Decimal256,
}

impl From<NoriaMsg> for CosmosMsg<NoriaMsg> {
    fn from(msg: NoriaMsg) -> CosmosMsg<NoriaMsg> {
        CosmosMsg::Custom(msg)
    }
}

impl CustomMsg for NoriaMsg {}
impl CustomQuery for NoriaQuery {}

// query_chain queries for any availabe query in the chain native modules
pub fn query_noria(deps: Deps, request: &QueryRequest<NoriaQuery>) -> StdResult<Binary> {
    let raw = to_vec(request).map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    })?;
    match deps.querier.raw_query(&raw) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::QueryRequest;

    use crate::msg::{AllianceQuery, NoriaQuery};

    #[test]
    fn test_alliance_params_query() {
        let request = QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::Params {}));
        let serialized = serde_json::to_string_pretty(&request).unwrap();

        println!("{}", serialized);
    }
}
