use std::error::Error;

use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        message::Message,
        signature::{read_keypair_file, Keypair, Signature},
        signer::Signer,
        system_instruction::create_account,
        transaction::Transaction,
    },
    Client,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    solana_program::pubkey::Pubkey,
    ID as TOKEN_ID,
};

pub fn create_token_mint(
    client: &RpcClient,
    payer: &Keypair,
    mint: &Keypair,
    mint_authority: &Pubkey,
    token_program_id: &Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let create_account_ix = create_account(
        &payer.pubkey(),
        &mint.pubkey(),
        client.get_minimum_balance_for_rent_exemption(82)?,
        82,
        &token_program_id,
    );
    let intialize_ix = initialize_mint(&token_program_id, &mint.pubkey(), mint_authority, None, 6)?;
    let message = Message::new(&[create_account_ix, intialize_ix], Some(&payer.pubkey()));
    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new(&[payer, &mint], message, blockhash);
    let sig = client.send_and_confirm_transaction(&tx)?;
    Ok(sig)
}

pub fn mint_token(
    client: &RpcClient,
    payer: &Keypair,
    mint: &Pubkey,
    amount: u64,
    token_program_id: &Pubkey,
) -> Result<Signature, Box<dyn Error>> {
    let user_ata = get_associated_token_address(&payer.pubkey(), mint);
    let create_ata_ix =
        create_associated_token_account(&payer.pubkey(), &payer.pubkey(), mint, token_program_id);
    let mint_ix = mint_to(&TOKEN_ID, mint, &user_ata, &payer.pubkey(), &[], amount).unwrap();
    let message = Message::new(&[create_ata_ix, mint_ix], Some(&payer.pubkey()));
    let tx = Transaction::new(&[&payer], message, client.get_latest_blockhash().unwrap());
    let sig = client.send_and_confirm_transaction(&tx).unwrap();
    Ok(sig)
}

#[cfg(test)]
mod tests {
    use std::{cmp::min, thread::sleep, time::Duration};

    use super::*;
    #[test]
    pub fn test_create_token_mint() {
        let client = RpcClient::new("http://localhost:8899");
        let path = "/Users/daiwanwei/.config/solana/id.json";
        let payer = read_keypair_file(&path).expect("invalid payer keypair file");
        let mint = Keypair::new();
        let blockhash = client.get_latest_blockhash().unwrap();

        println!("{:?}", payer.pubkey().to_string());
        println!("{:?}", client.get_balance(&payer.pubkey()).unwrap());
        let sig = client.request_airdrop(&payer.pubkey(), 50000).unwrap();
        println!("{:?}", sig);
        let bal = client.get_balance(&payer.pubkey()).unwrap();
        println!("{:?}", bal);
        let res = create_token_mint(&client, &payer, &mint, &payer.pubkey(), &TOKEN_ID).unwrap();
    }

    #[test]
    fn test_mint_token() {
        let client = RpcClient::new("http://localhost:8899");
        let path = "/Users/daiwanwei/.config/solana/id.json";
        let payer = read_keypair_file(&path).expect("invalid payer keypair file");
        let mint = Keypair::new();
        let blockhash = client.get_latest_blockhash().unwrap();
        let res = create_token_mint(&client, &payer, &mint, &payer.pubkey(), &TOKEN_ID).unwrap();
        let amount = 1000000;
        let mint_res = mint_token(&client, &payer, &mint.pubkey(), amount, &TOKEN_ID).unwrap();
        println!("{:?}", mint_res);
    }
}
