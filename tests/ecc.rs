use std::str::FromStr;

use walletcryptography::ru256::RU256;
use walletcryptography::secp256k1::*;
use walletcryptography::base16;

use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::prelude::*;

#[test]
#[ignore]
fn ecc() {
    // generate a random private key
    let mut rng = rand::thread_rng();
    let bytes = [0; 32];
    let random_bytes = bytes.into_iter().map(|_| rng.gen_range(0..=255) ).collect::<Vec<u8>>();
    let pr_n = hex::encode(random_bytes);

    // generate public key with custom-wrote curve arithmetics
    let pub_key1 = SECP256K1::pr_to_pub(&RU256::from_str(&pr_n).unwrap());
    let pub_key_str1 = pub_key1.to_hex_string();

    // generate public key with production library
    let secp = Secp256k1::new();
    let pr_key = SecretKey::from_str(&pr_n).expect("private-key");
    let pub_key2 = PublicKey::from_secret_key(&secp, &pr_key);
    let pub_key_str2 = base16::encode_bytes(&pub_key2.serialize_uncompressed());

    assert_eq!(pub_key_str1, pub_key_str2);
}

