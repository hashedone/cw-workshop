use crate::msg::InstantiateMsg;
use crate::state::{Config, CONFIG, MEMBERS};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult};

pub const PEER_INSTANTIATE_ID: u64 = 1;

pub fn instantiate(deps: DepsMut, msg: InstantiateMsg) -> StdResult<Response> {
    let config = Config {
        peer_code_id: msg.peer_code_id,
        incremental_donation: msg.incremental_donation,
        collective_ratio: msg.collective_ratio,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

pub mod exec {
    use cosmwasm_std::{to_binary, Addr, BankMsg, Order, SubMsg, SubMsgResult, Uint128, WasmMsg};
    use cw_utils::parse_instantiate_response_data;
    use donation_peer::msg::{
        DonatorsResp, InstantiateMsg as PeerInstantiate, QueryMsg as PeerQuery,
    };

    use crate::state::PENDING_INSTANTIATION;

    use super::*;

    pub fn join(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let sender = info.sender.to_string();
        let config = CONFIG.load(deps.storage)?;

        let msg = PeerInstantiate {
            owner: sender.clone(),
            incremental_donation: config.incremental_donation,
            collective_ratio: config.collective_ratio,
        };

        let msg = WasmMsg::Instantiate {
            admin: None,
            code_id: config.peer_code_id,
            msg: to_binary(&msg)?,
            funds: vec![],
            label: format!("peer-{}", sender),
        };

        PENDING_INSTANTIATION.save(deps.storage, &info.sender)?;

        let resp = Response::new()
            .add_submessage(SubMsg::reply_on_success(msg, PEER_INSTANTIATE_ID))
            .add_attribute("action", "join")
            .add_attribute("sender", sender);

        Ok(resp)
    }

    pub fn peer_instantiate_reply(deps: DepsMut, msg: SubMsgResult) -> StdResult<Response> {
        let resp = match msg.into_result() {
            Ok(resp) => resp,
            Err(err) => return Err(StdError::generic_err(err)),
        };

        let data = resp
            .data
            .ok_or_else(|| StdError::generic_err("No instantiate response data"))?;
        let resp = parse_instantiate_response_data(&data)
            .map_err(|err| StdError::generic_err(err.to_string()))?;

        let addr = PENDING_INSTANTIATION.load(deps.storage)?;
        let peer = Addr::unchecked(&resp.contract_address);

        MEMBERS.save(deps.storage, peer, &addr)?;

        Ok(Response::new())
    }

    pub fn leave(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let member = MEMBERS
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|addr| addr.ok())
            .find(|(_, addr)| *addr == info.sender);

        if let Some((peer, _)) = member {
            MEMBERS.remove(deps.storage, peer);
        } else {
            return Err(StdError::generic_err(
                "No such member or state read failure",
            ));
        }

        Ok(Response::new())
    }

    pub fn donate(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        let weights: Vec<_> = MEMBERS
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|peer| -> StdResult<_> {
                let peer = peer?;
                let donations_resp: DonatorsResp = deps
                    .querier
                    .query_wasm_smart(peer.clone(), &PeerQuery::Donators {})?;
                Ok((peer, donations_resp.donators as u128))
            })
            .collect::<StdResult<_>>()?;

        let total: u128 = weights.iter().map(|(_, weight)| weight).sum();

        let funds = deps.querier.query_all_balances(env.contract.address)?;
        let send_msgs = weights.into_iter().map(|(peer, weights)| {
            let coins: Vec<_> = funds
                .iter()
                .cloned()
                .map(|mut coin| {
                    coin.amount = Uint128::new(coin.amount.u128() * weights / total);
                    coin
                })
                .collect();

            BankMsg::Send {
                to_address: peer.to_string(),
                amount: coins,
            }
        });

        let resp = Response::new()
            .add_messages(send_msgs)
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.to_string());

        Ok(resp)
    }
}

pub mod query {
    use cw_storage_plus::Bound;

    use super::*;

    use crate::msg::{ConfigResp, Member, MemberPeerAddrResp, MembersListResp};

    pub fn config(deps: Deps) -> StdResult<ConfigResp> {
        let config = CONFIG.load(deps.storage)?;
        Ok(ConfigResp {
            incremental_donation: config.incremental_donation,
            collective_ratio: config.collective_ratio,
        })
    }

    pub fn member_peer_addr(deps: Deps, addr: &str) -> StdResult<MemberPeerAddrResp> {
        let peer = MEMBERS
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|addr| addr.ok())
            .find(|(_, member)| member.as_str() == addr);

        let (peer, _) = peer.ok_or_else(|| StdError::generic_err("No such member"))?;

        Ok(MemberPeerAddrResp { addr: peer })
    }

    pub fn members_list(
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u64>,
    ) -> StdResult<MembersListResp> {
        let start_after = start_after
            .map(|addr| deps.api.addr_validate(&addr))
            .transpose()?;

        let members = MEMBERS
            .range(
                deps.storage,
                start_after.map(Bound::exclusive),
                None,
                Order::Ascending,
            )
            .map(|member| -> StdResult<_> {
                let (peer, addr) = member?;

                Ok(Member {
                    addr,
                    peer_addr: peer,
                })
            });

        let members: Vec<_> = if let Some(limit) = limit {
            members.take(limit as usize).collect::<StdResult<_>>()
        } else {
            members.collect()
        }?;

        Ok(MembersListResp { members })
    }
}
