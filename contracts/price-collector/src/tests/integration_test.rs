#![cfg(test)]
mod tests {
    use crate::msg::ExecuteMsg;
    use crate::tests::env_setup::env::{
        instantiate_contracts, ADMIN, CONTROLLER, CONTROLLER_FAKE, USER1,
    };
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use price_feed::msg::{
        ExecuteMsg as PriceFeedExecuteMsg, QueryMsg as PriceFeedQueryMsg, RoundDataResponse,
    };

    mod price_feed_testing {
        use super::*;

        #[test]
        fn only_owner_can_change_controller() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            let price_feed_contract_addr = &contracts[0].contract_addr;

            // prepare the update controller message
            let update_controller_msg = PriceFeedExecuteMsg::UpdateController {
                controller: CONTROLLER_FAKE.to_string(),
            };

            let res = app.execute_contract(
                Addr::unchecked(USER1),
                Addr::unchecked(price_feed_contract_addr),
                &update_controller_msg,
                &[],
            );
            assert!(res.is_err());

            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(price_feed_contract_addr),
                &update_controller_msg,
                &[],
            );
            assert!(res.is_ok());
        }

        #[test]
        fn only_controller_can_update_answer() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            let price_feed_contract_addr = &contracts[0].contract_addr;

            // prepare the update controller message
            let update_answer_msg = PriceFeedExecuteMsg::UpdateRoundData { answer: 100000u64 };

            let res = app.execute_contract(
                Addr::unchecked(USER1),
                Addr::unchecked(price_feed_contract_addr),
                &update_answer_msg,
                &[],
            );
            assert!(res.is_err());

            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(price_feed_contract_addr),
                &update_answer_msg,
                &[],
            );
            assert!(res.is_err());

            let res = app.execute_contract(
                Addr::unchecked(CONTROLLER),
                Addr::unchecked(price_feed_contract_addr),
                &update_answer_msg,
                &[],
            );
            assert!(res.is_ok());

            // query the price feed contract to check if the answer is updated
            let query_msg = PriceFeedQueryMsg::LastestRoundData {};

            let res: RoundDataResponse = app
                .wrap()
                .query_wasm_smart(price_feed_contract_addr, &query_msg)
                .unwrap();
            assert_eq!(res.answer, Uint128::from(100000u64));
        }
    }

    mod price_collector_testing {
        use super::*;

        #[test]
        fn admin_can_provide_round_rata() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            let price_collector_contract_addr = &contracts[1].contract_addr;

            // prepare the update round data message
            let provide_round_data_msg = ExecuteMsg::ProvideRoundData { answer: 100000u64 };

            let res = app.execute_contract(
                Addr::unchecked(USER1),
                Addr::unchecked(price_collector_contract_addr),
                &provide_round_data_msg,
                &[],
            );
            assert!(res.is_err());

            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(price_collector_contract_addr),
                &provide_round_data_msg,
                &[],
            );
            println!("{:?}", res);
            assert!(res.is_ok());
        }
    }
}
