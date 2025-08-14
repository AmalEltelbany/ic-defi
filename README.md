# DeFi AMM - Decentralized Exchange on Internet Computer

>Automated Market Maker built on the Internet Computer Protocol, enabling seamless decentralized trading and liquidity provision.

![Internet Computer](https://img.shields.io/badge/Internet%20Computer-3B00B9?style=flat-square&logo=internet-computer&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)

## Overview

This AMM enables decentralized token trading on the Internet Computer without the need for order books. Users can trade tokens instantly through liquidity pools, while liquidity providers earn fees from every transaction.

The system implements the constant product formula (x × y = k) to automatically determine prices based on supply and demand, creating an efficient and fair trading environment.

## System Architecture

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#e1f5fe', 'primaryTextColor': '#01579b', 'primaryBorderColor': '#0288d1', 'lineColor': '#0288d1', 'secondaryColor': '#f3e5f5', 'tertiaryColor': '#e8f5e8', 'background': '#ffffff', 'mainBkg': '#e3f2fd', 'secondBkg': '#f1f8e9', 'tertiaryBkg': '#fce4ec'}}}%%
graph TB
    subgraph "Internet Computer Network"
        subgraph "AMM System"
            AMM[AMM Canister]
            LedgerA[Token A Ledger<br/>ICRC-1/ICRC-2]
            LedgerB[Token B Ledger<br/>ICRC-1/ICRC-2]
        end
        
        subgraph "Users"
            LP[Liquidity Providers]
            Traders[Token Traders]
            Depositors[Vault Users]
        end
    end
    
    LP -->|Add Liquidity| AMM
    LP <-->|Remove Liquidity| AMM
    Traders -->|Token Swaps| AMM
    Depositors <-->|Deposit/Withdraw| AMM
    
    AMM <-->|Transfer Operations| LedgerA
    AMM <-->|Transfer Operations| LedgerB
```

## How It Works

### Liquidity Pools
Liquidity providers deposit pairs of tokens into pools. These pools serve as the source of liquidity for all trades. In return, providers receive LP tokens representing their share of the pool.

### Automated Market Making
The system uses the constant product formula where the product of the two token reserves remains constant during trades:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#fff3e0', 'primaryTextColor': '#e65100', 'primaryBorderColor': '#ff9800', 'lineColor': '#ff9800', 'secondaryColor': '#e8f5e8', 'tertiaryColor': '#f3e5f5'}}}%%
graph LR
    A[Token A Reserve: X] --> K[Constant Product: K = X × Y]
    B[Token B Reserve: Y] --> K
    K --> Price[Automatic Price Discovery]
```

### Trading Process
When a user wants to swap Token A for Token B:

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#e8f5e8', 'primaryTextColor': '#2e7d32', 'primaryBorderColor': '#4caf50', 'lineColor': '#4caf50', 'actorBkg': '#c8e6c9', 'actorTextColor': '#1b5e20', 'actorLineColor': '#388e3c', 'signalColor': '#4caf50', 'signalTextColor': '#1b5e20'}}}%%
sequenceDiagram
    participant User
    participant AMM
    participant LedgerA
    participant LedgerB
    
    User->>AMM: Initiate swap request
    AMM->>LedgerA: Transfer Token A from user
    AMM->>AMM: Calculate output amount
    AMM->>LedgerB: Transfer Token B to user
    AMM->>AMM: Update pool reserves
```

## Core Features

### Token Swapping
Users can instantly exchange one token for another with a 0.3% trading fee. The system includes slippage protection through minimum output amount specifications.

### Liquidity Provision
Users can deposit token pairs to earn LP tokens and receive a proportional share of all trading fees. Liquidity can be removed at any time, returning the original tokens plus accumulated fees.

### Vault System
A secure deposit and withdrawal system that tracks user balances internally for improved gas efficiency while maintaining full security through ICRC-2 approvals.

### Reserve Management
Real-time tracking of token reserves ensures accurate pricing and liquidity information is always available.


## Mathematical Foundation

### Constant Product Formula
The core pricing mechanism follows: **x × y = k**

Where:
- x = Token A reserve
- y = Token B reserve  
- k = Constant product

### Swap Calculation
When swapping amount `dx` of Token A for Token B:

```
dy = y × dx × 0.997 / (x + dx × 0.997)
```

The 0.997 factor represents the 0.3% trading fee.

### LP Token Valuation
For liquidity provision:
- Initial LP tokens: `sqrt(amount_a × amount_b)`
- Additional LP tokens: `min(amount_a × total_lp / reserve_a, amount_b × total_lp / reserve_b)`

## Candid Functions

### Trading Operations
- `swap(token_in, amount_in, min_amount_out)` - Execute token exchanges
- `get_reserves()` - Retrieve current pool reserves

### Liquidity Management
- `add_liquidity(amount_a, amount_b)` - Provide liquidity to pools
- `remove_liquidity(lp_amount)` - Remove liquidity from pools
- `get_lp_balance()` - Check LP token balance
- `get_total_lp()` - Get total LP token supply

### Vault Operations
- `deposit(amount)` - Deposit tokens to personal vault
- `withdraw(amount, to_account)` - Withdraw tokens from vault
- `balance()` - Check vault balance
- `transfer(amount, to_account)` - Transfer tokens between accounts

