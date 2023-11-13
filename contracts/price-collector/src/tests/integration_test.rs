#![cfg(test)]
mod tests {
    use crate::contract::MAX_DIFF_DURRATION;
    use crate::msg::ExecuteMsg;
    use crate::tests::env_setup::env::{instantiate_contracts, ADMIN, USER1};
    use cosmwasm_std::{Addr, Uint128};
    use cw_multi_test::Executor;
    use price_feed::msg::{QueryMsg as PriceFeedQueryMsg, RoundDataResponse};

    mod price_collector_testing {
        use cosmwasm_std::BlockInfo;

        use super::*;

        #[test]
        fn admin_can_provide_round_rata() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            let price_feed_contract_addr = &contracts[0].contract_addr;
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
            assert!(res.is_ok());

            // increase the block height to simulate the time passed
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(100001),
                height: app.block_info().height + MAX_DIFF_DURRATION + 100,
                chain_id: app.block_info().chain_id,
            });

            let res = app.execute_contract(
                Addr::unchecked(ADMIN),
                Addr::unchecked(price_collector_contract_addr),
                &provide_round_data_msg,
                &[],
            );
            assert!(res.is_ok());

            // query the price in price feed contract to check if the answer is updated
            let query_msg = PriceFeedQueryMsg::LastestRoundData {};

            let res: RoundDataResponse = app
                .wrap()
                .query_wasm_smart(price_feed_contract_addr, &query_msg)
                .unwrap();
            assert_eq!(res.answer, Uint128::from(100000u64));
        }
    }
}
