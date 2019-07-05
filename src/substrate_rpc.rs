// Copyright 2019 PolkaWorld.

use codec::{Compact, Decode, Encode};
use futures::Future;
use hex;
use runtime::{AccountId, Hash, Nonce};
use runtime::{Address, Call, Runtime};
use serde_json::Value;
use sr_primitives::generic::Era;
use srml_support::storage::StorageMap;
use substrate_primitives::crypto::Pair as TraitPair;
use substrate_primitives::hexdisplay::HexDisplay;
use substrate_primitives::{blake2_256, sr25519::Pair};
use Rpc;

pub fn account_pair(s: &str) -> Pair {
    Pair::from_string(&format!("//{}", s), None).expect("static values are valid; qed")
}

pub fn genesis_hash(client: &mut Rpc) -> Hash {
    client
        .request::<Hash>("chain_getBlockHash", vec![json!(0 as u64)])
        .wait()
        .unwrap()
        .unwrap()
}

pub fn account_balance(client: &mut Rpc, account_id: &AccountId) {
    let key = <srml_balances::FreeBalance<Runtime>>::key_for(account_id);
    let key = blake2_256(&key);
    let key = format!("0x{:}", HexDisplay::from(&key));
    let balance = client
        .request::<Value>("state_getStorage", vec![json!(key)])
        .wait()
        .unwrap()
        .unwrap();
    if balance.is_string() {
        let balance = balance.as_str().unwrap();
        let blob = hex::decode(&balance[2..]).unwrap();
        let balance: Option<u128> = Decode::decode(&mut blob.as_slice());
        println!("account:{:?}, balance:{:?}", account_id, balance);
    } else {
        println!("account:{:?}, balance:{:?}", account_id, balance);
    }
}

pub fn account_nonce(client: &mut Rpc, account_id: &AccountId) -> Nonce {
    let key = <srml_system::AccountNonce<Runtime>>::key_for(account_id);
    let key = blake2_256(&key);
    let key = format!("0x{:}", HexDisplay::from(&key));
    let nonce = client
        .request::<Value>("state_getStorage", vec![json!(key)])
        .wait()
        .unwrap()
        .unwrap();
    if nonce.is_string() {
        let nonce = nonce.as_str().unwrap();
        let blob = hex::decode(&nonce[2..]).unwrap();
        let index: Option<Nonce> = Decode::decode(&mut blob.as_slice());
        index.unwrap()
    } else {
        0
    }
}

pub fn put_transaction(client: &mut Rpc, tx: String) -> u64 {
    client
        .request::<u64>("author_submitAndWatchExtrinsic", vec![json!(tx)])
        .wait()
        .unwrap()
        .unwrap()
}

pub fn generate_transfer_tx(
    pair: &Pair,
    from: AccountId,
    index: Nonce,
    hash: Hash,
    to: AccountId,
) -> String {
    let func = runtime::Call::Balances(runtime::BalancesCall::transfer::<runtime::Runtime>(
        to.into(),
        10000 as u128,
    ));

    generate_tx(pair, from, func, index, (Era::Immortal, hash))
}

pub fn generate_payment_tx(
    pair: &Pair,
    from: AccountId,
    index: Nonce,
    hash: Hash,
) -> String {
    let func = runtime::Call::Demo(runtime::DemoCall::set_payment::<runtime::Runtime>(
        10000 as u128,
    ));

    generate_tx(pair, from, func, index, (Era::Immortal, hash))
}

pub fn generate_play_tx(
    pair: &Pair,
    from: AccountId,
    index: Nonce,
    hash: Hash,
) -> String {
    let func = runtime::Call::Demo(runtime::DemoCall::play::<runtime::Runtime>());

    generate_tx(pair, from, func, index, (Era::Immortal, hash))
}

pub fn generate_sudo_tx(
    pair: &Pair,
    from: AccountId,
    index: Nonce,
    hash: Hash,
    new: Vec<u8>,
) -> String {
    let func = runtime::Call::Sudo(runtime::SudoCall::sudo::<runtime::Runtime>(Box::new(
        runtime::Call::System(runtime::SystemCall::set_code::<runtime::Runtime>(new)),
    )));

    generate_tx(pair, from, func, index, (Era::Immortal, hash))
}

fn generate_tx(
    pair: &Pair,
    sender: AccountId,
    function: Call,
    index: Nonce,
    e: (Era, Hash),
) -> String {
    let era = e.0;
    let hash: Hash = e.1;
    let sign_index: Compact<Nonce> = index.into();
    let signed: Address = sender.into();
    let signer = pair.clone();

    let raw_payload = (sign_index, function.clone(), era, hash);
    let signature = raw_payload.using_encoded(|payload| {
        if payload.len() > 256 {
            signer.sign(&blake2_256(payload)[..])
        } else {
            signer.sign(payload)
        }
    });

    /*let signature = sr25519::Signature(s);
    assert_eq!(
        sr_primitives::verify_encoded_lazy(&signature, &payload, &sender),
        true
    );*/

    // 编码字段 1 元组(发送人，签名)，func | 签名：(index,func, era, h)
    let uxt = runtime::UncheckedExtrinsic::new_signed(index, function, signed, signature, era);
    let t: Vec<u8> = uxt.encode();
    //format!("0x{:}", HexDisplay::from(&t))
    format!("0x{:}", hex::encode(&t))
}
