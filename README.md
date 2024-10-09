# EasyCoin

**EasyCoin** is the first AI-powered trading platform on Solana, composed of an on-chain Program and an off-chain AI Agent. 

This repository is the on-chain Program. 
The on-chain Program enhances fund security through DeFi mechanisms, managing assets and executing trades. 
Compared to existing Solana trading bots, EasyCoin offers the following **security advantages**:
- **No Need for User’s Private Key**: Funds are managed by a Program Derived Address (PDA), eliminating the need for users to share their private keys.
- **User-Controlled Withdrawals**: Only users can withdraw their funds; the AI Agent is limited to executing trades based on user instructions.
- **Multisig Wallet Management**: The Program is managed with multisig wallets, avoiding the risks associated with a single admin key.
- **72-Hour Timelock**: Any changes to the program are subject to a 72-hour timelock, adding an extra layer of security.

## Program (Smart Contract) Addresses
The Easycoin Program is deployed to:
- Solana Mainnet-beta: `easyTwKoYFtBTzmNqGYjKS5nZ9SvdTkhPxSHbBMnraY`

## Compiling the code
You can compile the code with Anchor.
``` Bash
anchor build
```
Or, you can produce a verifiable build by running
``` Bash
anchor build --verifiable
```

If you do not have the Solana Anchor framework CLI installed, you can do so by following [this guide](https://www.anchor-lang.com/docs/installation).

## Verifying the code
You can verify that the on-chain program binary is indeed compiled from the source code in this repository by following these steps:

First, clone this repository and checkout the commit hash.
``` Bash
git clone https://github.com/RealEasyCoin/easycoin-program.git
git checkout 46d438ab52da794779f23f69dec4f5a8c97b4dcf
```
Then, verify the program with the following command.
``` Bash
# SOLANA_RPC_URL is the URL of the Solana rpc node, e.g. https://api.mainnet-beta.solana.com
anchor verify -p easycoin easyTwKoYFtBTzmNqGYjKS5nZ9SvdTkhPxSHbBMnraY --provider.cluster SOLANA_RPC_URL 
```

If the verification is successful, it confirms that the on-chain program matches the source code in this repository.

## Security
EasyCoin has undergone a comprehensive security audit by BlockSec, a leading blockchain security firm. 
You can view the full audit report at the following link: 

[BlockSec Audit Report for EasyCoin](https://github.com/blocksecteam/audit-reports/blob/main/rust/blocksec_easycoin_v1.0-signed.pdf)