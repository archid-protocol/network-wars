#![cfg(test)]
use cosmwasm_std::{
    Addr, Coin, Uint128,
};
use cw_multi_test::Executor;

use crate::integration_tests::util::{
    bank_query, create_netwars, mint_native, mock_app, query,
};

use crate::msg::{
    ExecuteMsg, QueryMsg,
};
use crate::contract::DENOM;
use crate::state::{State};

// Valid deposits must increase the timer,  
// invalid deposits must return an error
#[test]
fn test_deposit() {
    let mut app = mock_app();
    
    // netwars owner deploys netwars
    let netwars_admin = Addr::unchecked("netwars_deployer");
    // depositor owns ARCH
    let depositor = Addr::unchecked("arch_owner");

    // mint native to depositor
    mint_native(
        &mut app,
        depositor.to_string(),
        Uint128::from(100000000000000000000_u128), // 100 ARCH as aarch
    );

    // contract settings
    let expiration: u64 = 604800; // ~1 week
    let min_deposit =  Uint128::from(1000000000000000000_u128); // 1 ARCH as aarch
    let extension_length: u64 = 3600; // 1 hour 
    let stale: u64 = 604800; // ~1 week
    let reset_length: u64 = 604800; // ~1 week

    // netwars_admin creates the netwars contract 
    let netwars_addr: Addr = create_netwars(
        &mut app, 
        &netwars_admin, 
        None,
        None,
        expiration.clone(), 
        min_deposit.clone(),
        extension_length.clone(),
        stale,
        reset_length,
        &[],
    );

    // contract balance (netwars prize) is currently 0
    let netwars_balance: Coin = bank_query(&mut app, &netwars_addr);
    assert_eq!(netwars_balance.amount, Uint128::from(0_u128));

    // depositor must send at least the min_deposit amount
    assert!(
        app.execute_contract(
            depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Deposit{}, 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(1000_u128) // Invalid amount (less than min_deposit)
            }]
        ).is_err()
    );

    // depositing a valid amount must increase the game timer
    let initial_game_state: State = query(
        &mut app,
        netwars_addr.clone(),
        QueryMsg::Game{},
    ).unwrap();
    let _res = app
        .execute_contract(
            depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Deposit{}, 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(1000000000000000000_u128)
            }]
        )
        .unwrap();
    let game_query: State = query(
        &mut app,
        netwars_addr.clone(),
        QueryMsg::Game{},
    ).unwrap();
    let expected_expiration: u64 = initial_game_state.expiration + extension_length;
    assert_eq!(game_query.expiration, expected_expiration);

    // prize pool must be increased
    let netwars_balance: Coin = bank_query(&mut app, &netwars_addr);
    assert_eq!(netwars_balance.amount, Uint128::from(1000000000000000000_u128));
}