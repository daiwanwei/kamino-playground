mod kamino;
mod pyth;
mod token;
use std::error::Error;

use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        account, bs58,
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        system_instruction::create_account,
        system_program::ID as SYSTEM_ID,
        sysvar::rent::ID as SYSVAR_RENT_ID,
    },
    Client, Cluster,
};
use anchor_spl::token::ID as TOKEN_ID;
use kamino_lend::{accounts, instruction, state::LendingMarket, ID as KAMINO_LENDING_ID};

use crate::kamino::create_lending_market;
fn main() -> Result<(), Box<dyn Error>> {
    let path = "/Users/daiwanwei/.config/solana/id.json";
    let payer = read_keypair_file(&path).expect("invalid payer keypair file");

    let client: Client<&Keypair> =
        Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program = client.program(KAMINO_LENDING_ID)?;
    let rpc_client = program.rpc();
    let balance = rpc_client.get_balance(&payer.pubkey())?;
    println!("balance: {}", balance);

    let state = program.account::<LendingMarket>(Pubkey::try_from(
        "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
    )?)?;
    println!("{:?}", bs58::encode(state.quote_currency).into_string());
    let lending_market = Keypair::new();
    let size = std::mem::size_of::<LendingMarket>() as u64 + 8;
    println!("size: {}", size);
    let size = 4664;
    println!("size: {}", size);
    let res = create_lending_market(&program, &payer, &lending_market).unwrap();
    println!("{:?}", res);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
