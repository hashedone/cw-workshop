mod contract;
pub mod msg;
pub mod state;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: msg::InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
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

#[cfg_attr(not(feature = "library"), entry_point)]
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        contract::PEER_INSTANTIATE_ID => contract::exec::peer_instantiate_reply(deps, msg.result),
        _ => Err(StdError::generic_err("unknown reply id")),
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{
        ConfigResp, ExecMsg, InstantiateMsg, MemberPeerAddrResp, MembersListResp, QueryMsg,
    };
    use peer::msg::{DonatorsResp, ExecMsg as PeerExec, ManagerResp, QueryMsg as PeerQuery};

    use super::*;

    use cosmwasm_std::{coin, coins, Addr, Decimal, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use donation_peer as peer;
    use peer::msg::OwnerResp;

    fn peer() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(peer::execute, peer::instantiate, peer::query);
        Box::new(contract)
    }

    fn manager() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    #[test]
    fn instantiate_check() {
        let mut app = App::default();
        let peer_code_id = app.store_code(peer());
        let manager_code_id = app.store_code(manager());

        let manager = app
            .instantiate_contract(
                manager_code_id,
                Addr::unchecked("admin"),
                &InstantiateMsg {
                    peer_code_id,
                    incremental_donation: coin(100, "utgd"),
                    collective_ratio: Decimal::percent(60),
                },
                &[],
                "manager",
                None,
            )
            .unwrap();

        let config: ConfigResp = app
            .wrap()
            .query_wasm_smart(manager.clone(), &QueryMsg::Config {})
            .unwrap();

        assert_eq!(coin(100, "utgd"), config.incremental_donation);
        assert_eq!(Decimal::percent(60), config.collective_ratio);

        let members: MembersListResp = app
            .wrap()
            .query_wasm_smart(
                manager,
                &QueryMsg::MembersList {
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();

        assert!(members.members.is_empty());
    }

    #[test]
    fn join() {
        let mut app = App::default();
        let peer_code_id = app.store_code(peer());
        let manager_code_id = app.store_code(manager());

        let manager = app
            .instantiate_contract(
                manager_code_id,
                Addr::unchecked("admin"),
                &InstantiateMsg {
                    peer_code_id,
                    incremental_donation: coin(100, "utgd"),
                    collective_ratio: Decimal::percent(60),
                },
                &[],
                "manager",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("member"),
            manager.clone(),
            &ExecMsg::Join {},
            &[],
        )
        .unwrap();

        let peer: MemberPeerAddrResp = app
            .wrap()
            .query_wasm_smart(
                manager.clone(),
                &QueryMsg::MemberPeerAddr {
                    addr: "member".to_owned(),
                },
            )
            .unwrap();

        let owner: OwnerResp = app
            .wrap()
            .query_wasm_smart(peer.addr.clone(), &PeerQuery::Owner {})
            .unwrap();

        assert_eq!(Addr::unchecked("member"), owner.owner);

        let manager_resp: ManagerResp = app
            .wrap()
            .query_wasm_smart(peer.addr.clone(), &PeerQuery::Manager {})
            .unwrap();

        assert_eq!(manager, manager_resp.manager);

        let donators: DonatorsResp = app
            .wrap()
            .query_wasm_smart(peer.addr, &PeerQuery::Donators {})
            .unwrap();

        assert_eq!(0, donators.donators);
    }

    #[test]
    fn single_peer_single_donate() {
        // After a single donation, the only peer should got the whole amount

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("donator"), coins(100, "utgd"))
                .unwrap();
        });
        let peer_code_id = app.store_code(peer());
        let manager_code_id = app.store_code(manager());

        let manager = app
            .instantiate_contract(
                manager_code_id,
                Addr::unchecked("admin"),
                &InstantiateMsg {
                    peer_code_id,
                    incremental_donation: coin(100, "utgd"),
                    collective_ratio: Decimal::percent(60),
                },
                &[],
                "manager",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("member"),
            manager.clone(),
            &ExecMsg::Join {},
            &[],
        )
        .unwrap();

        let peer: MemberPeerAddrResp = app
            .wrap()
            .query_wasm_smart(
                manager,
                &QueryMsg::MemberPeerAddr {
                    addr: "member".to_owned(),
                },
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("donator"),
            peer.addr.clone(),
            &PeerExec::Donate {},
            &coins(100, "utgd"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("member"),
            peer.addr.clone(),
            &PeerExec::Withdraw {},
            &[],
        )
        .unwrap();

        assert_eq!(
            coin(0, "utgd"),
            app.wrap().query_balance("donator", "utgd").unwrap()
        );
        assert_eq!(
            coin(0, "utgd"),
            app.wrap()
                .query_balance(peer.addr.as_str(), "utgd")
                .unwrap()
        );
        assert_eq!(
            coin(100, "utgd"),
            app.wrap().query_balance("member", "utgd").unwrap()
        );
    }

    #[test]
    fn two_peers_single_donate_per_peer() {
        // After the first donations all funds (100utgd) goes to the peer1
        // After second donation:
        // * 40% of funds stays on the peer2 (40utgd)
        // * 60% of funds are split between peer1 and peer2 (60utgd - 30utgd each)
        // As the result the peer1 should receive 130utgd, and the peer2 should have
        // 70utgd

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("donator"), coins(200, "utgd"))
                .unwrap();
        });
        let peer_code_id = app.store_code(peer());
        let manager_code_id = app.store_code(manager());

        let manager = app
            .instantiate_contract(
                manager_code_id,
                Addr::unchecked("admin"),
                &InstantiateMsg {
                    peer_code_id,
                    incremental_donation: coin(100, "utgd"),
                    collective_ratio: Decimal::percent(60),
                },
                &[],
                "manager",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("member1"),
            manager.clone(),
            &ExecMsg::Join {},
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("member2"),
            manager.clone(),
            &ExecMsg::Join {},
            &[],
        )
        .unwrap();

        let peer1: MemberPeerAddrResp = app
            .wrap()
            .query_wasm_smart(
                manager.clone(),
                &QueryMsg::MemberPeerAddr {
                    addr: "member1".to_owned(),
                },
            )
            .unwrap();

        let peer2: MemberPeerAddrResp = app
            .wrap()
            .query_wasm_smart(
                manager,
                &QueryMsg::MemberPeerAddr {
                    addr: "member2".to_owned(),
                },
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("donator"),
            peer1.addr.clone(),
            &PeerExec::Donate {},
            &coins(100, "utgd"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("donator"),
            peer2.addr.clone(),
            &PeerExec::Donate {},
            &coins(100, "utgd"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("member1"),
            peer1.addr,
            &PeerExec::Withdraw {},
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("member2"),
            peer2.addr,
            &PeerExec::Withdraw {},
            &[],
        )
        .unwrap();

        assert_eq!(
            coin(130, "utgd"),
            app.wrap().query_balance("member1", "utgd").unwrap()
        );

        assert_eq!(
            coin(70, "utgd"),
            app.wrap().query_balance("member2", "utgd").unwrap()
        );
    }

    #[test]
    fn two_peers_single_donate_per_peer_under_threshold() {
        // After the first donations all funds (100utgd) goes to the peer1
        // Second donation is to small to increase counter
        // After second donation:
        // * 40% of funds stays on the peer2 (20utgd)
        // * 60% of funds goes to peer1 (30utgd)
        // As the result the peer1 should receive 130utgd, and the peer2 should have
        // 20utgd

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("donator"), coins(150, "utgd"))
                .unwrap();
        });
        let peer_code_id = app.store_code(peer());
        let manager_code_id = app.store_code(manager());

        let manager = app
            .instantiate_contract(
                manager_code_id,
                Addr::unchecked("admin"),
                &InstantiateMsg {
                    peer_code_id,
                    incremental_donation: coin(100, "utgd"),
                    collective_ratio: Decimal::percent(60),
                },
                &[],
                "manager",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("member1"),
            manager.clone(),
            &ExecMsg::Join {},
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("member2"),
            manager.clone(),
            &ExecMsg::Join {},
            &[],
        )
        .unwrap();

        let peer1: MemberPeerAddrResp = app
            .wrap()
            .query_wasm_smart(
                manager.clone(),
                &QueryMsg::MemberPeerAddr {
                    addr: "member1".to_owned(),
                },
            )
            .unwrap();

        let peer2: MemberPeerAddrResp = app
            .wrap()
            .query_wasm_smart(
                manager,
                &QueryMsg::MemberPeerAddr {
                    addr: "member2".to_owned(),
                },
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("donator"),
            peer1.addr.clone(),
            &PeerExec::Donate {},
            &coins(100, "utgd"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("donator"),
            peer2.addr.clone(),
            &PeerExec::Donate {},
            &coins(50, "utgd"),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("member1"),
            peer1.addr,
            &PeerExec::Withdraw {},
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("member2"),
            peer2.addr,
            &PeerExec::Withdraw {},
            &[],
        )
        .unwrap();

        assert_eq!(
            coin(130, "utgd"),
            app.wrap().query_balance("member1", "utgd").unwrap()
        );

        assert_eq!(
            coin(20, "utgd"),
            app.wrap().query_balance("member2", "utgd").unwrap()
        );
    }
}
