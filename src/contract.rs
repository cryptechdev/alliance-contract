#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{AllianceQuery, ExecuteMsg, InstantiateMsg, NoriaMsg};
use crate::state::{State, STATE};

use self::execute::{claim_rewards, delegate, redelegate, undelegate};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:alliance";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NoriaMsg>, ContractError> {
    match msg {
        ExecuteMsg::Delegate {
            validator_address,
            amount,
        } => delegate(info, validator_address, amount),
        ExecuteMsg::Undelegate {
            validator_address,
            amount,
        } => undelegate(info, validator_address, amount),
        ExecuteMsg::Redelegate {
            validator_src_address,
            validator_dst_address,
            amount,
        } => redelegate(info, validator_src_address, validator_dst_address, amount),
        ExecuteMsg::ClaimDelegationRewards {
            validator_address,
            denom,
        } => claim_rewards(info, validator_address, denom),
    }
}

pub mod execute {
    use cosmwasm_std::Coin;

    use crate::msg::{AllianceMsg, NoriaMsg};

    use super::*;

    pub fn delegate(
        info: MessageInfo,
        validator_address: Addr,
        amount: Coin,
    ) -> Result<Response<NoriaMsg>, ContractError> {
        let msg = NoriaMsg::Alliance(AllianceMsg::Delegate {
            delegator_address: info.sender,
            validator_address,
            amount,
        });

        let res = Response::new()
            .add_attribute("method", "delegate")
            .add_attribute("module", "alliance")
            .add_message(msg);

        Ok(res)
    }
    pub fn undelegate(
        info: MessageInfo,
        validator_address: Addr,
        amount: Coin,
    ) -> Result<Response<NoriaMsg>, ContractError> {
        let msg = NoriaMsg::Alliance(AllianceMsg::Undelegate {
            delegator_address: info.sender,
            validator_address,
            amount,
        });

        let res = Response::new()
            .add_attribute("method", "undelegate")
            .add_attribute("module", "alliance")
            .add_message(msg);

        Ok(res)
    }
    pub fn redelegate(
        info: MessageInfo,
        validator_src_address: Addr,
        validator_dst_address: Addr,
        amount: Coin,
    ) -> Result<Response<NoriaMsg>, ContractError> {
        let msg = NoriaMsg::Alliance(AllianceMsg::Redelegate {
            delegator_address: info.sender,
            validator_src_address,
            validator_dst_address,
            amount,
        });

        let res = Response::new()
            .add_attribute("method", "redelegate")
            .add_attribute("module", "alliance")
            .add_message(msg);

        Ok(res)
    }
    pub fn claim_rewards(
        info: MessageInfo,
        validator_address: Addr,
        denom: String,
    ) -> Result<Response<NoriaMsg>, ContractError> {
        let msg = NoriaMsg::Alliance(AllianceMsg::ClaimDelegationRewards {
            delegator_address: info.sender,
            validator_address,
            denom,
        });

        let res = Response::new()
            .add_attribute("method", "claim_rewards")
            .add_attribute("module", "alliance")
            .add_message(msg);

        Ok(res)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: AllianceQuery) -> StdResult<Binary> {
    match msg {
        AllianceQuery::Params {} => to_binary(&query::get_params(deps)?),
        AllianceQuery::Alliance { denom } => to_binary(&query::get_alliance(deps, denom)?),
        AllianceQuery::Alliances { pagination } => {
            to_binary(&query::get_alliances(deps, pagination)?)
        }
        AllianceQuery::AlliancesDelegations { pagination } => {
            to_binary(&query::get_alliances_delegations(deps, pagination)?)
        }
        AllianceQuery::AlliancesDelegationByValidator {
            delegator_addr,
            validator_addr,
            pagination,
        } => to_binary(&query::get_alliances_delegations_by_validator(
            deps,
            delegator_addr,
            validator_addr,
            pagination,
        )?),
        AllianceQuery::Delegation {
            delegator_addr,
            validator_addr,
            denom,
        } => to_binary(&query::get_delegation(
            deps,
            delegator_addr,
            validator_addr,
            denom,
        )?),
        AllianceQuery::DelegationRewards {
            delegator_addr,
            validator_addr,
            denom,
        } => to_binary(&query::get_delegation_rewards(
            deps,
            delegator_addr,
            validator_addr,
            denom,
        )?),
        AllianceQuery::Validator { validator_addr } => {
            to_binary(&query::get_validator(deps, validator_addr)?)
        }
        AllianceQuery::Validators { pagination } => {
            to_binary(&query::get_validators(deps, pagination)?)
        }
    }
}

pub mod query {
    use cosmwasm_std::{from_binary, QueryRequest};

    use crate::msg::{
        query_noria, AllValidatorsResponse, AllianceAllianceResponse,
        AllianceAlliancesDelegationsResponse, AllianceAlliancesResponse, AllianceParamsResponse,
        AllianceQuery, NoriaQuery, Pagination, RewardsResponse, SingleDelegationResponse,
        ValidatorResponse,
    };

    use super::*;

    pub fn get_params(deps: Deps) -> StdResult<AllianceParamsResponse> {
        let request = QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::Params {}));
        let response: AllianceParamsResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<AllianceParamsResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }

    pub(crate) fn get_alliance(deps: Deps, denom: String) -> StdResult<AllianceAllianceResponse> {
        let request = QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::Alliance { denom }));
        let response: AllianceAllianceResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<AllianceAllianceResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }

    pub(crate) fn get_alliances(
        deps: Deps,
        pagination: Option<Pagination>,
    ) -> StdResult<AllianceAlliancesResponse> {
        let request = QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::Alliances {
            pagination,
        }));
        let response: AllianceAlliancesResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<AllianceAlliancesResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }

    pub(crate) fn get_alliances_delegations(
        deps: Deps,
        pagination: Option<Pagination>,
    ) -> StdResult<AllianceAlliancesDelegationsResponse> {
        let request =
            QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::AlliancesDelegations {
                pagination,
            }));
        let response: AllianceAlliancesDelegationsResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<AllianceAlliancesDelegationsResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }

    pub(crate) fn get_alliances_delegations_by_validator(
        deps: Deps,
        delegator_addr: Addr,
        validator_addr: Addr,
        pagination: Option<Pagination>,
    ) -> StdResult<AllianceAlliancesDelegationsResponse> {
        let request = QueryRequest::Custom(NoriaQuery::Alliance(
            AllianceQuery::AlliancesDelegationByValidator {
                delegator_addr,
                validator_addr,
                pagination,
            },
        ));
        let response: AllianceAlliancesDelegationsResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<AllianceAlliancesDelegationsResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }

    pub(crate) fn get_delegation(
        deps: Deps,
        delegator_addr: Addr,
        validator_addr: Addr,
        denom: String,
    ) -> StdResult<SingleDelegationResponse> {
        let request = QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::Delegation {
            delegator_addr,
            validator_addr,
            denom,
        }));
        let response: SingleDelegationResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<SingleDelegationResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }

    pub(crate) fn get_delegation_rewards(
        deps: Deps,
        delegator_addr: Addr,
        validator_addr: Addr,
        denom: String,
    ) -> StdResult<RewardsResponse> {
        let request =
            QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::DelegationRewards {
                delegator_addr,
                validator_addr,
                denom,
            }));
        let response: RewardsResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<RewardsResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }

    pub(crate) fn get_validator(deps: Deps, validator_addr: Addr) -> StdResult<ValidatorResponse> {
        let request = QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::Validator {
            validator_addr,
        }));
        let response: ValidatorResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<ValidatorResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }

    pub(crate) fn get_validators(
        deps: Deps,
        pagination: Option<Pagination>,
    ) -> StdResult<AllValidatorsResponse> {
        let request = QueryRequest::Custom(NoriaQuery::Alliance(AllianceQuery::Validators {
            pagination,
        }));
        let response: AllValidatorsResponse;
        match query_noria(deps, &request) {
            Err(err) => {
                return Err(err);
            }
            Ok(binary) => {
                match from_binary::<AllValidatorsResponse>(&binary) {
                    Err(_) => {
                        return Err(cosmwasm_std::StdError::GenericErr {
                            msg: binary.to_string(),
                        });
                    }
                    Ok(res) => response = res,
                };
            }
        }

        Ok(response)
    }
}
