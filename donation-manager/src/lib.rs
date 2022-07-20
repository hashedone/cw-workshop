mod contract;
pub mod msg;
pub mod state;

#[cfg(not(feature = "library"))]
mod entry_points {
    use cosmwasm_std::{
        entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
        StdResult,
    };

    use crate::{contract, msg};

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: msg::InstantiateMsg,
    ) -> StdResult<Response> {
        contract::instantiate(deps, msg)
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
            Join {} => exec::join(deps, info),
            Leave {} => exec::leave(deps, info),
            Donate {} => exec::donate(deps, env, info),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
        use contract::query;
        use msg::QueryMsg::*;

        match msg {
            Config {} => to_binary(&query::config(deps)?),
            MemberPeerAddr { addr } => to_binary(&query::member_peer_addr(deps, &addr)?),
            MembersList { start_after, limit } => {
                to_binary(&query::members_list(deps, start_after, limit)?)
            }
        }
    }

    #[entry_point]
    pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
        match msg.id {
            contract::PEER_INSTANTIATE_ID => {
                contract::exec::peer_instantiate_reply(deps, msg.result)
            }
            _ => Err(StdError::generic_err("unknown reply id")),
        }
    }
}

#[cfg(not(feature = "library"))]
pub use entry_points::*;
