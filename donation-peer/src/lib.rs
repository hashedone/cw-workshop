mod contract;
pub mod msg;
pub mod state;

#[cfg(not(feature = "library"))]
mod entry_points {
    use cosmwasm_std::{
        entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    };

    use crate::{contract, msg};

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: msg::InstantiateMsg,
    ) -> StdResult<Response> {
        contract::instantiate(deps, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: msg::ExecMsg,
    ) -> StdResult<Response> {
        use contract::exec;
        use msg::ExecMsg::*;

        match msg {
            Donate {} => exec::donate(deps, info),
            Withdraw {} => exec::withdraw(deps, env, info),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
        use contract::query;
        use msg::QueryMsg::*;

        match msg {
            Owner {} => to_binary(&query::owner(deps)?),
            Manager {} => to_binary(&query::manager(deps)?),
            PendingDonations { denom } => to_binary(&query::pending_donations(deps, env, denom)?),
            Config {} => to_binary(&query::config(deps)?),
            Donators {} => to_binary(&query::donators(deps)?),
        }
    }
}

#[cfg(not(feature = "library"))]
pub use entry_points::*;
