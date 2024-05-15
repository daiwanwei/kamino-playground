use pyth_sdk_solana::{load_price_feed_from_account, Price, PriceFeed};
#[cfg(test)]
mod tests {
    use std::{
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    };

    use anchor_client::{
        anchor_lang::prelude::*,
        solana_client::rpc_client::RpcClient,
        solana_sdk::{account::Account, pubkey::Pubkey, signature::read_keypair_file},
    };

    use super::*;

    #[test]
    fn test_pyth() {
        let path = "/Users/daiwanwei/.config/solana/id.json";
        let payer = read_keypair_file(&path).expect("invalid payer keypair file");
        let url = "http://api.devnet.solana.com";
        let clnt = RpcClient::new(url);
        const STALENESS_THRESHOLD: u64 = 60; // staleness threshold in seconds
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let price_key: Pubkey =
            Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();
        let mut price_account: Account = clnt.get_account(&price_key).unwrap();
        let price_feed: PriceFeed =
            load_price_feed_from_account(&price_key, &mut price_account).unwrap();
        println!("{:?}", price_feed);
        let current_price: Price =
            price_feed.get_price_no_older_than(current_time, STALENESS_THRESHOLD).unwrap();
        println!(
            "price: ({} +- {}) x 10^{}",
            current_price.price, current_price.conf, current_price.expo
        );
    }
}
