use crate::msg::{InstantiateMsg, ManagerExec};
use crate::state::{State, OWNER, STATE};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult};

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    let owner = deps.api.addr_validate(&msg.owner)?;
    OWNER.save(deps.storage, &owner)?;

    let state = State {
        donators: 0,
        incremental_donation: msg.incremental_donation,
        collective_ratio: msg.collective_ratio,
        manager: info.sender,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new())
}

pub mod exec {
    use cosmwasm_std::{to_binary, BankMsg, StdError, WasmMsg};

    use super::*;

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let mut state = STATE.load(deps.storage)?;
        let increment = info.funds.iter().any(|coin| {
            coin.denom == state.incremental_donation.denom
                && coin.amount >= state.incremental_donation.amount
        });

        if increment {
            state.donators += 1;
            STATE.save(deps.storage, &state)?;
        }

        let collective_donation: Vec<_> = info
            .funds
            .into_iter()
            .map(|mut coin| {
                coin.amount = coin.amount * state.collective_ratio;
                coin
            })
            .collect();

        let donate_msg = ManagerExec::Donate {};
        let donate_msg = WasmMsg::Execute {
            contract_addr: state.manager.to_string(),
            msg: to_binary(&donate_msg)?,
            funds: collective_donation,
        };

        let resp = Response::new()
            .add_message(donate_msg)
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender)
            .add_attribute("donators_increment", if increment { "yes" } else { "no" });

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(StdError::generic_err("Unauthorized"));
        }

        let donations = deps.querier.query_all_balances(env.contract.address)?;

        let resp = Response::new()
            .add_message(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: donations,
            })
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender);

        Ok(resp)
    }
}

pub mod query {
    use super::*;

    use crate::msg::{ConfigResp, DonatorsResp, ManagerResp, OwnerResp, PendingDonationsResp};

    pub fn owner(deps: Deps) -> StdResult<OwnerResp> {
        let owner = OWNER.load(deps.storage)?;
        Ok(OwnerResp { owner })
    }

    pub fn manager(deps: Deps) -> StdResult<ManagerResp> {
        let state = STATE.load(deps.storage)?;
        Ok(ManagerResp {
            manager: state.manager,
        })
    }

    pub fn pending_donations(
        deps: Deps,
        env: Env,
        denom: Option<String>,
    ) -> StdResult<PendingDonationsResp> {
        let donations = if let Some(denom) = denom {
            vec![deps.querier.query_balance(env.contract.address, denom)?]
        } else {
            deps.querier.query_all_balances(env.contract.address)?
        };

        Ok(PendingDonationsResp { donations })
    }

    pub fn config(deps: Deps) -> StdResult<ConfigResp> {
        let state = STATE.load(deps.storage)?;
        Ok(ConfigResp {
            incremental_donation: state.incremental_donation,
            collective_ratio: state.collective_ratio,
        })
    }

    pub fn donators(deps: Deps) -> StdResult<DonatorsResp> {
        let state = STATE.load(deps.storage)?;
        Ok(DonatorsResp {
            donators: state.donators,
        })
    }
}
