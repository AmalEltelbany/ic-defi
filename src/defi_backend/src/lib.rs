use candid::{CandidType, Deserialize, Principal, Nat};
use ic_cdk::api::{canister_self, msg_caller};
use ic_cdk::api::call::call;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{BlockIndex, NumTokens, TransferArg as Icrc1TransferArgs, TransferError};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::cmp::min;
use once_cell::sync::Lazy;

thread_local! {
    static VAULT_BALANCES: RefCell<HashMap<Account, NumTokens>> = RefCell::new(HashMap::new());
    static RESERVE_A: RefCell<NumTokens> = RefCell::new(NumTokens::from(0u64));
    static RESERVE_B: RefCell<NumTokens> = RefCell::new(NumTokens::from(0u64));
    static TOTAL_LP: RefCell<NumTokens> = RefCell::new(NumTokens::from(0u64));
    static LP_BALANCES: RefCell<HashMap<Account, NumTokens>> = RefCell::new(HashMap::new());
}

static LEDGER_A: Lazy<Principal> = Lazy::new(|| {
    Principal::from_text("uxrrr-q7777-77774-qaaaq-cai").expect("Could not decode the principal.")
});
static LEDGER_B: Lazy<Principal> = Lazy::new(|| {
    Principal::from_text("uzt4z-lp777-77774-qaabq-cai").expect("Could not decode the principal.")
});

fn integer_sqrt(n: &Nat) -> Nat {
    let zero = Nat::from(0u64);
    let one = Nat::from(1u64);
    if n == &zero || n == &one {
        return n.clone();
    }
    let mut low = one.clone();
    let mut high = (n.clone() / Nat::from(2u64)) + one.clone();
    while low < high {
        let mid = (low.clone() + high.clone()) / Nat::from(2u64) + one.clone();
        if mid.clone() * mid.clone() > *n {
            high = mid - one.clone();
        } else {
            low = mid;
        }
    }
    low
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct TransferArgs {
    amount: NumTokens,
    to_account: Account,
}

#[ic_cdk::update]
async fn transfer(args: TransferArgs) -> Result<BlockIndex, String> {
    ic_cdk::println!(
        "Transferring {} tokens to account {}",
        &args.amount,
        &args.to_account
    );

    let transfer_from_args = TransferFromArgs {
        from: Account::from(msg_caller()),
        memo: None,
        amount: args.amount,
        spender_subaccount: None,
        fee: None,
        to: args.to_account,
        created_at_time: None,
    };

    let result: Result<(Result<BlockIndex, TransferFromError>,), _> = call(*LEDGER_A, "icrc2_transfer_from", (transfer_from_args,)).await;
    let transfer_result = result
        .map_err(|err| format!("failed to call ledger: {:?}", err))?
        .0;

    transfer_result.map_err(|e| format!("ledger transfer error: {:?}", e))
}

#[ic_cdk::update]
async fn deposit(amount: NumTokens) -> Result<BlockIndex, String> {
    let caller_account = Account::from(msg_caller());
    let canister_account = Account {
        owner: canister_self(),
        subaccount: None,
    };

    let transfer_from_args = TransferFromArgs {
        from: caller_account,
        to: canister_account,
        amount: amount.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
        spender_subaccount: None,
    };

    let result: Result<(Result<BlockIndex, TransferFromError>,), _> = call(*LEDGER_A, "icrc2_transfer_from", (transfer_from_args,)).await;
    let transfer_result = result
        .map_err(|err| format!("failed to call ledger: {:?}", err))?
        .0;

    let block_index = transfer_result.map_err(|e| format!("ledger transfer error: {:?}", e))?;

 
    VAULT_BALANCES.with(|balances_ref| {
        let mut balances = balances_ref.borrow_mut();
        let entry = balances.entry(caller_account).or_insert(NumTokens::from(0u64));
        *entry = entry.clone() + amount;
    });

    Ok(block_index)
}

#[ic_cdk::query]
fn balance() -> NumTokens {
    let caller_account = Account::from(msg_caller());
    VAULT_BALANCES.with(|balances_ref| {
        balances_ref
            .borrow()
            .get(&caller_account)
            .cloned()
            .unwrap_or(NumTokens::from(0u64))
    })
}

#[ic_cdk::update]
async fn withdraw(amount: NumTokens, to_account: Account) -> Result<BlockIndex, String> {
    let caller_account = Account::from(msg_caller());

    // Check internal balance
    let user_balance = VAULT_BALANCES.with(|balances_ref| {
        balances_ref
            .borrow()
            .get(&caller_account)
            .cloned()
            .unwrap_or(NumTokens::from(0u64))
    });

    if user_balance < amount {
        return Err("insufficient balance".to_string());
    }

    let transfer_args = Icrc1TransferArgs {
        from_subaccount: None,
        to: to_account,
        amount: amount.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let result: Result<(Result<BlockIndex, TransferError>,), _> = call(*LEDGER_A, "icrc1_transfer", (transfer_args,)).await;
    let transfer_result = result
        .map_err(|err| format!("failed to call ledger: {:?}", err))?
        .0;

    let block_index = transfer_result.map_err(|e| format!("ledger transfer error: {:?}", e))?;

    // Update internal balance on success
    VAULT_BALANCES.with(|balances_ref| {
        let mut balances = balances_ref.borrow_mut();
        if let Some(entry) = balances.get_mut(&caller_account) {
            *entry = entry.clone() - amount;
        }
    });

    Ok(block_index)
}

#[ic_cdk::update]
async fn add_liquidity(amount_a: NumTokens, amount_b: NumTokens) -> Result<NumTokens, String> {
    if amount_a == 0u64 || amount_b == 0u64 {
        return Err("Amounts must be positive".to_string());
    }

    let caller = Account::from(msg_caller());
    let canister = Account { owner: canister_self(), subaccount: None };

    // Transfer A from caller to canister
    let tf_args_a = TransferFromArgs {
        from: caller,
        to: canister,
        amount: amount_a.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
        spender_subaccount: None,
    };
    
    let result_a: Result<(Result<BlockIndex, TransferFromError>,), _> = call(*LEDGER_A, "icrc2_transfer_from", (tf_args_a,)).await;
    let _result_a = result_a
        .map_err(|err| format!("failed to call ledger A: {:?}", err))?
        .0
        .map_err(|e| format!("ledger A transfer error: {:?}", e))?;

    // Transfer B from caller to canister
    let tf_args_b = TransferFromArgs {
        from: caller,
        to: canister,
        amount: amount_b.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
        spender_subaccount: None,
    };
    
    let result_b: Result<(Result<BlockIndex, TransferFromError>,), _> = call(*LEDGER_B, "icrc2_transfer_from", (tf_args_b,)).await;
    let _result_b = result_b
        .map_err(|err| format!("failed to call ledger B: {:?}", err))?
        .0
        .map_err(|e| format!("ledger B transfer error: {:?}", e))?;

    let reserve_a_old = RESERVE_A.with(|r| r.borrow().clone());
    let reserve_b_old = RESERVE_B.with(|r| r.borrow().clone());
    let total_lp_old = TOTAL_LP.with(|t| t.borrow().clone());

    let lp_to_mint: NumTokens;
    if total_lp_old == 0u64 {
        lp_to_mint = integer_sqrt(&(amount_a.clone() * amount_b.clone()));
    } else {
        let lp_from_a = amount_a.clone() * total_lp_old.clone() / reserve_a_old.clone();
        let lp_from_b = amount_b.clone() * total_lp_old.clone() / reserve_b_old.clone();
        lp_to_mint = min(lp_from_a, lp_from_b);
    }

    if lp_to_mint == 0u64 {
        return Err("Zero LP to mint".to_string());
    }

    // Update reserves
    RESERVE_A.with(|r| *r.borrow_mut() = reserve_a_old + amount_a);
    RESERVE_B.with(|r| *r.borrow_mut() = reserve_b_old + amount_b);

    // Update total LP
    TOTAL_LP.with(|t| *t.borrow_mut() = total_lp_old + lp_to_mint.clone());

    // Update caller's LP balance
    LP_BALANCES.with(|balances_ref| {
        let mut balances = balances_ref.borrow_mut();
        let entry = balances.entry(caller).or_insert(NumTokens::from(0u64));
        *entry = entry.clone() + lp_to_mint.clone();
    });

    Ok(lp_to_mint)
}

#[ic_cdk::update]
async fn remove_liquidity(lp_amount: NumTokens) -> Result<(NumTokens, NumTokens), String> {
    if lp_amount == 0u64 {
        return Err("Amount must be positive".to_string());
    }

    let caller = Account::from(msg_caller());

    let user_lp = LP_BALANCES.with(|balances_ref| {
        balances_ref
            .borrow()
            .get(&caller)
            .cloned()
            .unwrap_or(NumTokens::from(0u64))
    });

    if user_lp < lp_amount {
        return Err("Insufficient LP balance".to_string());
    }

    let reserve_a = RESERVE_A.with(|r| r.borrow().clone());
    let reserve_b = RESERVE_B.with(|r| r.borrow().clone());
    let total_lp = TOTAL_LP.with(|t| t.borrow().clone());

    if total_lp == 0u64 {
        return Err("No liquidity".to_string());
    }

    let amount_a_out = lp_amount.clone() * reserve_a.clone() / total_lp.clone();
    let amount_b_out = lp_amount.clone() * reserve_b.clone() / total_lp.clone();

    if amount_a_out == 0u64 || amount_b_out == 0u64 {
        return Err("Zero amounts out".to_string());
    }

    // Transfer A to caller
    let t_args_a = Icrc1TransferArgs {
        from_subaccount: None,
        to: caller,
        amount: amount_a_out.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    
    let result_a: Result<(Result<BlockIndex, TransferError>,), _> = call(*LEDGER_A, "icrc1_transfer", (t_args_a,)).await;
    let _result_a = result_a
        .map_err(|err| format!("failed to call ledger A: {:?}", err))?
        .0
        .map_err(|e| format!("ledger A transfer error: {:?}", e))?;

    // Transfer B to caller
    let t_args_b = Icrc1TransferArgs {
        from_subaccount: None,
        to: caller,
        amount: amount_b_out.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    
    let result_b: Result<(Result<BlockIndex, TransferError>,), _> = call(*LEDGER_B, "icrc1_transfer", (t_args_b,)).await;
    let _result_b = result_b
        .map_err(|err| format!("failed to call ledger B: {:?}", err))?
        .0
        .map_err(|e| format!("ledger B transfer error: {:?}", e))?;

    // Update reserves
    RESERVE_A.with(|r| *r.borrow_mut() = reserve_a - amount_a_out.clone());
    RESERVE_B.with(|r| *r.borrow_mut() = reserve_b - amount_b_out.clone());

    // Update total LP
    TOTAL_LP.with(|t| *t.borrow_mut() = total_lp - lp_amount.clone());

    // Update caller's LP balance
    LP_BALANCES.with(|balances_ref| {
        let mut balances = balances_ref.borrow_mut();
        if let Some(entry) = balances.get_mut(&caller) {
            *entry = entry.clone() - lp_amount;

        }
    });

    Ok((amount_a_out, amount_b_out))
}

#[ic_cdk::update]
async fn swap(token_in: Principal, amount_in: NumTokens, min_amount_out: NumTokens) -> Result<NumTokens, String> {
    if amount_in == 0u64 {
        return Err("Amount must be positive".to_string());
    }

    let caller = Account::from(msg_caller());
    let canister = Account { owner: canister_self(), subaccount: None };

    let (ledger_in, ledger_out, reserve_in_cell, reserve_out_cell) = if token_in == *LEDGER_A {
        (*LEDGER_A, *LEDGER_B, &RESERVE_A, &RESERVE_B)
    } else if token_in == *LEDGER_B {
        (*LEDGER_B, *LEDGER_A, &RESERVE_B, &RESERVE_A)
    } else {
        return Err("Invalid token".to_string());
    };

    // Transfer in from caller to canister
    let tf_args = TransferFromArgs {
        from: caller,
        to: canister,
        amount: amount_in.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
        spender_subaccount: None,
    };
    
    let result_in: Result<(Result<BlockIndex, TransferFromError>,), _> = call(ledger_in, "icrc2_transfer_from", (tf_args,)).await;
    let _result_in = result_in
        .map_err(|err| format!("failed to call ledger in: {:?}", err))?
        .0
        .map_err(|e| format!("ledger in transfer error: {:?}", e))?;

    let reserve_in = reserve_in_cell.with(|r| r.borrow().clone());
    let reserve_out = reserve_out_cell.with(|r| r.borrow().clone());

    if reserve_in == 0u64 || reserve_out == 0u64 {
        return Err("No liquidity".to_string());
    }

    let amount_in_fee = amount_in.clone() * Nat::from(997u64) / Nat::from(1000u64);
    let amount_out = reserve_out.clone() * amount_in_fee.clone() / (reserve_in.clone() + amount_in_fee);

    if amount_out == 0u64 {
        return Err("Zero amount out".to_string());
    }
    if amount_out < min_amount_out {
        return Err("Slippage tolerance exceeded".to_string());
    }

    // Transfer out to caller
    let t_args = Icrc1TransferArgs {
        from_subaccount: None,
        to: caller,
        amount: amount_out.clone(),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    
    let result_out: Result<(Result<BlockIndex, TransferError>,), _> = call(ledger_out, "icrc1_transfer", (t_args,)).await;
    let _result_out = result_out
        .map_err(|err| format!("failed to call ledger out: {:?}", err))?
        .0
        .map_err(|e| format!("ledger out transfer error: {:?}", e))?;

    // Update reserves
    reserve_in_cell.with(|r| *r.borrow_mut() = reserve_in + amount_in);
    reserve_out_cell.with(|r| *r.borrow_mut() = reserve_out - amount_out.clone());

    Ok(amount_out)
}

#[ic_cdk::query]
fn get_reserves() -> (NumTokens, NumTokens) {
    (
        RESERVE_A.with(|r| r.borrow().clone()),
        RESERVE_B.with(|r| r.borrow().clone()),
    )
}

#[ic_cdk::query]
fn get_lp_balance() -> NumTokens {
    let caller = Account::from(msg_caller());
    LP_BALANCES.with(|balances_ref| {
        balances_ref
            .borrow()
            .get(&caller)
            .cloned()
            .unwrap_or(NumTokens::from(0u64))
    })
}

#[ic_cdk::query]
fn get_total_lp() -> NumTokens {
    TOTAL_LP.with(|t| t.borrow().clone())
}

ic_cdk::export_candid!();