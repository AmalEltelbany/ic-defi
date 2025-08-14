#!/bin/bash

# Get the principal of the current identity
PRINCIPAL=$(dfx identity get-principal)
echo "Using principal: $PRINCIPAL"

# Stop any running local network
dfx stop

# Start fresh local network
dfx start --clean --background

# Deploy Token A Ledger
echo "Deploying Token A Ledger..."
dfx deploy token_a_ledger --argument "(variant { Init = record { 
    token_name = \"Token A\"; 
    token_symbol = \"TKNA\"; 
    decimals = opt 8; 
    transfer_fee = 10_000 : nat; 
    minting_account = record { 
        owner = principal \"$PRINCIPAL\"; 
        subaccount = null 
    }; 
    initial_balances = vec { 
        record { 
            record { 
                owner = principal \"$PRINCIPAL\"; 
                subaccount = null 
            }; 
            1_000_000_000_000 : nat 
        } 
    }; 
    archive_options = record { 
        trigger_threshold = 2000 : nat64; 
        num_blocks_to_archive = 1000 : nat64; 
        controller_id = principal \"$PRINCIPAL\" 
    }; 
    metadata = vec { 
        record { \"icrc1:name\"; variant { Text = \"Token A\" } }; 
        record { \"icrc1:symbol\"; variant { Text = \"TKNA\" } } 
    }; 
    fee_collector_account = null; 
    max_memo_length = opt 32 : opt nat16; 
    feature_flags = opt record { icrc2 = true }; 
    maximum_number_of_accounts = opt 100_000 : opt nat64; 
    accounts_overflow_trim_quantity = opt 100 : opt nat64 
} })"

# Deploy Token B Ledger
echo "Deploying Token B Ledger..."
dfx deploy token_b_ledger --argument "(variant { Init = record { 
    token_name = \"Token B\"; 
    token_symbol = \"TKNB\"; 
    decimals = opt 8; 
    transfer_fee = 10_000 : nat; 
    minting_account = record { 
        owner = principal \"$PRINCIPAL\"; 
        subaccount = null 
    }; 
    initial_balances = vec { 
        record { 
            record { 
                owner = principal \"$PRINCIPAL\"; 
                subaccount = null 
            }; 
            1_000_000_000_000 : nat 
        } 
    }; 
    archive_options = record { 
        trigger_threshold = 2000 : nat64; 
        num_blocks_to_archive = 1000 : nat64; 
        controller_id = principal \"$PRINCIPAL\" 
    }; 
    metadata = vec { 
        record { \"icrc1:name\"; variant { Text = \"Token B\" } }; 
        record { \"icrc1:symbol\"; variant { Text = \"TKNB\" } } 
    }; 
    fee_collector_account = null; 
    max_memo_length = opt 32 : opt nat16; 
    feature_flags = opt record { icrc2 = true }; 
    maximum_number_of_accounts = opt 100_000 : opt nat64; 
    accounts_overflow_trim_quantity = opt 100 : opt nat64 
} })"

# Get the canister IDs for the tokens
TOKEN_A_CANISTER=$(dfx canister id token_a_ledger)
TOKEN_B_CANISTER=$(dfx canister id token_b_ledger)

echo "Token A Canister ID: $TOKEN_A_CANISTER"
echo "Token B Canister ID: $TOKEN_B_CANISTER"

# Update the lib.rs file with the correct canister IDs
echo "Updating lib.rs with correct canister IDs..."

# Deploy the DeFi backend
echo "Deploying DeFi Backend..."
dfx deploy defi_backend

echo "All canisters deployed successfully!"
echo "Token A: $TOKEN_A_CANISTER"
echo "Token B: $TOKEN_B_CANISTER"
echo "DeFi Backend: $(dfx canister id defi_backend)"