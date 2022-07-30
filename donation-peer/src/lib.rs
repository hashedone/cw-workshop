pub mod contract;
pub mod msg;
pub mod state;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: msg::InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
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
        Donate {} => exec::donate(deps, info),
        Withdraw {} => exec::withdraw(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
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

#[cfg(test)]
mod tests {
    use crate::msg::{ConfigResp, DonatorsResp, ExecMsg};

    use super::*;
    use cosmwasm_std::{coin, coins, Addr, Decimal, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use msg::{ManagerResp, OwnerResp, PendingDonationsResp, QueryMsg};

    fn contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn instantiate_check() {
        let mut app = App::default();
        let code_id = app.store_code(contract());
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("manager"),
                &msg::InstantiateMsg {
                    owner: "owner".to_string(),
                    incremental_donation: coin(100, "utgd"),
                    collective_ratio: Decimal::percent(60),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let owner: OwnerResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::Owner {})
            .unwrap();
        assert_eq!(owner.owner, Addr::unchecked("owner"));

        let manager: ManagerResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::Manager {})
            .unwrap();
        assert_eq!(manager.manager, Addr::unchecked("manager"));

        let donations: PendingDonationsResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::PendingDonations { denom: None })
            .unwrap();
        assert_eq!(donations.donations, vec![]);

        let config: ConfigResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::Config {})
            .unwrap();
        assert_eq!(config.incremental_donation, coin(100, "utgd"));
        assert_eq!(config.collective_ratio, Decimal::percent(60));

        let donators: DonatorsResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::Donators {})
            .unwrap();
        assert_eq!(donators.donators, 0);
    }

    #[test]
    fn withdraw() {
        let mut app = App::default();
        let code_id = app.store_code(contract());
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("manager"),
                &msg::InstantiateMsg {
                    owner: "owner".to_string(),
                    incremental_donation: coin(100, "utgd"),
                    collective_ratio: Decimal::percent(60),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &addr, coins(500, "utgd"))
                .unwrap();
        });

        let donations: PendingDonationsResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::PendingDonations { denom: None })
            .unwrap();
        assert_eq!(donations.donations, coins(500, "utgd"));

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecMsg::Withdraw {},
            &[],
        )
        .unwrap();

        let donations: PendingDonationsResp = app
            .wrap()
            .query_wasm_smart(addr.clone(), &QueryMsg::PendingDonations { denom: None })
            .unwrap();
        assert_eq!(donations.donations, vec![]);

        assert_eq!(
            coin(0, "utgd"),
            app.wrap().query_balance(addr, "utgd").unwrap()
        );
        assert_eq!(
            coin(500, "utgd"),
            app.wrap()
                .query_balance(Addr::unchecked("owner"), "utgd")
                .unwrap()
        );
    }
}
