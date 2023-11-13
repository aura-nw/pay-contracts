#[cfg(test)]
pub mod env {
    use cosmwasm_std::{Addr, BlockInfo, Coin, Empty, Uint128};

    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use price_feed::contract::{
        execute as PriceFeedExecute, instantiate as PriceFeedInstantiate, query as PriceFeedQuery,
    };

    use price_feed::msg::{
        ExecuteMsg as PriceFeedExecuteMsg, InstantiateMsg as PriceFeedInstantiateMsg,
    };

    pub const ADMIN: &str = "aura1000000000000000000000000000000000admin";
    pub const USER1: &str = "aura1000000000000000000000000000000000user1";
    // pub const AURA: &str = "aura10000000000000000000000000000000000aura";
    pub const CONTROLLER: &str = "aura10000000000000000000000000000controller";

    pub const NATIVE_DENOM: &str = "uaura";
    pub const NATIVE_BALANCE: u128 = 1_000_000_000_000u128;

    pub const NATIVE_DENOM_2: &str = "utaura";
    pub const NATIVE_BALANCE_2: u128 = 1_000_000_000_000u128;

    pub struct ContractInfo {
        pub contract_addr: String,
        pub contract_code_id: u64,
    }

    // create app instance and init balance of NATIVE token for admin
    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE),
                        },
                        Coin {
                            denom: NATIVE_DENOM_2.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE_2),
                        },
                    ],
                )
                .unwrap();
        })
    }

    // create price feed contract
    pub fn price_feed_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(PriceFeedExecute, PriceFeedInstantiate, PriceFeedQuery);
        Box::new(contract)
    }

    // create price collector contract
    pub fn price_collector_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();
        // Create a vector to store all contract info ([halo factory - [0])
        let mut contract_info_vec: Vec<ContractInfo> = Vec::new();

        // store code of all contracts to the app and get the code ids
        let price_feed_contract_code_id = app.store_code(price_feed_contract_template());
        let price_collector_contract_code_id = app.store_code(price_collector_contract_template());

        // instantiate price feed contract
        let price_feed_contract_addr = app
            .instantiate_contract(
                price_feed_contract_code_id,
                Addr::unchecked(ADMIN),
                &PriceFeedInstantiateMsg {
                    controller: CONTROLLER.to_string(),
                    decimals: 6,
                    description: "AURA / VND".to_string(),
                },
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: price_feed_contract_addr.to_string(),
            contract_code_id: price_feed_contract_code_id,
        });

        // instantiate price collector contract
        let price_collector_contract_addr = app
            .instantiate_contract(
                price_collector_contract_code_id,
                Addr::unchecked(ADMIN),
                &InstantiateMsg {
                    price_feed: price_feed_contract_addr.to_string(),
                    decimals: 6,
                },
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: price_collector_contract_addr.to_string(),
            contract_code_id: price_collector_contract_code_id,
        });

        // update new price feeder for price collector contract
        let exec_msg = ExecuteMsg::UpdatePriceFeeder {
            price_feeder: ADMIN.to_string(),
            status: true,
        };
        let _res = app.execute_contract(
            Addr::unchecked(ADMIN),
            Addr::unchecked(price_collector_contract_addr.clone()),
            &exec_msg,
            &[],
        );

        // update new controller for price feed contract
        let exec_msg = PriceFeedExecuteMsg::UpdateController {
            controller: price_collector_contract_addr.to_string(),
        };
        let _res = app.execute_contract(
            Addr::unchecked(ADMIN),
            Addr::unchecked(price_feed_contract_addr),
            &exec_msg,
            &[],
        );

        // change block time increase 6 seconds to make phase active
        app.set_block(BlockInfo {
            time: app.block_info().time.plus_seconds(100000),
            height: app.block_info().height + 20,
            chain_id: app.block_info().chain_id,
        });

        // return the app instance and contract info vector
        (app, contract_info_vec)
    }

    #[test]
    fn test_instantiate_contracts() {
        let (_app, contract_info_vec) = instantiate_contracts();

        // check if all contracts are instantiated
        assert_eq!(contract_info_vec.len(), 2);
    }
}
