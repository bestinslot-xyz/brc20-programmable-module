use {
  base64::{engine::general_purpose, Engine},
  bitcoin::{
    absolute::LockTime,
    address::AddressData,
    blockdata::script,
    consensus::Decodable,
    consensus::Encodable,
    key::{Keypair, TapTweak},
    opcodes,
    psbt::Psbt,
    script::PushBytes,
    secp256k1::{self, schnorr::Signature, Message, Secp256k1, XOnlyPublicKey},
    sighash::{self, SighashCache, TapSighashType},
    transaction::Version,
    Address, Amount, EcdsaSighashType, OutPoint, PrivateKey, PublicKey, ScriptBuf, Sequence,
    Transaction, TxIn, TxOut, Witness,
  },
  error::Error,
  sha2::{Digest, Sha256},
  snafu::{ResultExt, Snafu},
  std::str::FromStr,
};

mod error;
mod sign;
mod util;
mod verify;

pub use {sign::*, util::*, verify::*};

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, rand::RngCore};

  // From https://github.com/bitcoin/bips/blob/master/bip-0322.mediawiki#test-vectors
  // and https://github.com/ACken2/bip322-js/blob/main/test/Verifier.test.ts
  // and https://github.com/bitcoin/bitcoin/blob/29b28d07fa958b89e1c7916fda5d8654474cf495/src/test/util_tests.cpp#L2747

  const WIF_PRIVATE_KEY: &str = "L3VFeEujGtevx9w18HD1fhRbCH67Az2dpCymeRE1SoPK6XQtaN2k";
  const SEGWIT_ADDRESS: &str = "bc1q9vza2e8x573nczrlzms0wvx3gsqjx7vavgkx0l";
  const TAPROOT_ADDRESS: &str = "bc1ppv609nr0vr25u07u95waq5lucwfm6tde4nydujnu8npg4q75mr5sxq8lt3";
  const LEGACY_ADDRESS: &str = "14vV3aCHBeStb5bkenkNHbe2YAFinYdXgc";

  const NESTED_SEGWIT_WIF_PRIVATE_KEY: &str =
    "KwTbAxmBXjoZM3bzbXixEr9nxLhyYSM4vp2swet58i19bw9sqk5z";
  const NESTED_SEGWIT_ADDRESS: &str = "3HSVzEhCFuH9Z3wvoWTexy7BMVVp3PjS6f";

  #[test]
  fn message_hashes_are_correct() {
    assert_eq!(
      hex::encode(tagged_hash(BIP322_TAG, "")),
      "c90c269c4f8fcbe6880f72a721ddfbf1914268a794cbb21cfafee13770ae19f1"
    );

    assert_eq!(
      hex::encode(tagged_hash(BIP322_TAG, "Hello World")),
      "f0eb03b1a75ac6d9847f55c624a99169b5dccba2a31f5b23bea77ba270de0a7a"
    );
  }

  #[test]
  fn to_spend_txids_correct() {
    assert_eq!(
      create_to_spend(
        &Address::from_str(SEGWIT_ADDRESS).unwrap().assume_checked(),
        ""
      )
      .unwrap()
      .compute_txid()
      .to_string(),
      "c5680aa69bb8d860bf82d4e9cd3504b55dde018de765a91bb566283c545a99a7"
    );

    assert_eq!(
      create_to_spend(
        &Address::from_str(SEGWIT_ADDRESS).unwrap().assume_checked(),
        "Hello World"
      )
      .unwrap()
      .compute_txid()
      .to_string(),
      "b79d196740ad5217771c1098fc4a4b51e0535c32236c71f1ea4d61a2d603352b"
    );
  }

  #[test]
  fn to_sign_txids_correct() {
    let to_spend = create_to_spend(
      &Address::from_str(SEGWIT_ADDRESS).unwrap().assume_checked(),
      "",
    )
    .unwrap();

    let to_sign = create_to_sign(&to_spend, None).unwrap();

    assert_eq!(
      to_sign.unsigned_tx.compute_txid().to_string(),
      "1e9654e951a5ba44c8604c4de6c67fd78a27e81dcadcfe1edf638ba3aaebaed6"
    );

    let to_spend = create_to_spend(
      &Address::from_str(SEGWIT_ADDRESS).unwrap().assume_checked(),
      "Hello World",
    )
    .unwrap();

    let to_sign = create_to_sign(&to_spend, None).unwrap();

    assert_eq!(
      to_sign.unsigned_tx.compute_txid().to_string(),
      "88737ae86f2077145f93cc4b153ae9a1cb8d56afa511988c149c5c8c9d93bddf"
    );
  }

  #[test]
  fn simple_verify_and_falsify_taproot() {
    assert!(
      verify::verify_simple_encoded(
        TAPROOT_ADDRESS,
        "Hello World", 
        "AUHd69PrJQEv+oKTfZ8l+WROBHuy9HKrbFCJu7U1iK2iiEy1vMU5EfMtjc+VSHM7aU0SDbak5IUZRVno2P5mjSafAQ=="
      ).is_ok()
    );

    assert_eq!(
      verify::verify_simple_encoded(
        TAPROOT_ADDRESS,
        "Hello World -- This should fail",
        "AUHd69PrJQEv+oKTfZ8l+WROBHuy9HKrbFCJu7U1iK2iiEy1vMU5EfMtjc+VSHM7aU0SDbak5IUZRVno2P5mjSafAQ=="
      ).unwrap_err().to_string(),
      "Invalid signature"
    );
  }

  #[test]
  fn simple_sign_taproot() {
    assert_eq!(
      sign::sign_simple_encoded(TAPROOT_ADDRESS, "Hello World", WIF_PRIVATE_KEY).unwrap(),
      "AUHd69PrJQEv+oKTfZ8l+WROBHuy9HKrbFCJu7U1iK2iiEy1vMU5EfMtjc+VSHM7aU0SDbak5IUZRVno2P5mjSafAQ=="
    );
  }

  #[test]
  fn roundtrip_taproot_simple() {
    assert!(verify::verify_simple_encoded(
      TAPROOT_ADDRESS,
      "Hello World",
      &sign::sign_simple_encoded(TAPROOT_ADDRESS, "Hello World", WIF_PRIVATE_KEY).unwrap()
    )
    .is_ok());
  }

  #[test]
  fn roundtrip_taproot_full() {
    assert!(verify::verify_full_encoded(
      TAPROOT_ADDRESS,
      "Hello World",
      &sign::sign_full_encoded(TAPROOT_ADDRESS, "Hello World", WIF_PRIVATE_KEY).unwrap()
    )
    .is_ok());
  }

  #[test]
  fn invalid_address() {
    assert_eq!(verify::verify_simple_encoded(
      LEGACY_ADDRESS,
      "",
      "AkcwRAIgM2gBAQqvZX15ZiysmKmQpDrG83avLIT492QBzLnQIxYCIBaTpOaD20qRlEylyxFSeEA2ba9YOixpX8z46TSDtS40ASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI=").unwrap_err().to_string(),
      format!("Unsuported address `{LEGACY_ADDRESS}`, only P2TR, P2WPKH and P2SH-P2WPKH allowed")
    )
  }

  #[test]
  fn signature_decode_error() {
    assert_eq!(
      verify::verify_simple_encoded(
        TAPROOT_ADDRESS,
        "Hello World",
        "invalid signature not in base64 encoding"
      )
      .unwrap_err()
      .to_string(),
      "Decode error for signature `invalid signature not in base64 encoding`"
    );

    assert_eq!(
      verify::verify_simple_encoded(
        TAPROOT_ADDRESS,
        "Hello World", 
        "AkcwRAIgM2gBAQqvZX15ZiysmKmQpDrG83avLIT492QBzLnQIxYCIBaTpOaD20qRlEylyxFSeEA2ba9YOixpX8z46TSDtS40ASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViH"
      ).unwrap_err().to_string(),
      "Decode error for signature `AkcwRAIgM2gBAQqvZX15ZiysmKmQpDrG83avLIT492QBzLnQIxYCIBaTpOaD20qRlEylyxFSeEA2ba9YOixpX8z46TSDtS40ASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViH`"
    )
  }

  #[test]
  fn simple_verify_and_falsify_p2wpkh() {
    assert!(
      verify::verify_simple_encoded(
        SEGWIT_ADDRESS,
        "Hello World",
        "AkcwRAIgZRfIY3p7/DoVTty6YZbWS71bc5Vct9p9Fia83eRmw2QCICK/ENGfwLtptFluMGs2KsqoNSk89pO7F29zJLUx9a/sASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI="
      ).is_ok()
    );

    assert!(
      verify::verify_simple_encoded(
        SEGWIT_ADDRESS,
        "Hello World - this should fail",
        "AkcwRAIgZRfIY3p7/DoVTty6YZbWS71bc5Vct9p9Fia83eRmw2QCICK/ENGfwLtptFluMGs2KsqoNSk89pO7F29zJLUx9a/sASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI="
      ).is_err()
    );

    assert!(
      verify::verify_simple_encoded(
        SEGWIT_ADDRESS,
        "Hello World",
        "AkgwRQIhAOzyynlqt93lOKJr+wmmxIens//zPzl9tqIOua93wO6MAiBi5n5EyAcPScOjf1lAqIUIQtr3zKNeavYabHyR8eGhowEhAsfxIAMZZEKUPYWI4BruhAQjzFT8FSFSajuFwrDL1Yhy"
      ).is_ok()
    );

    assert!(
      verify::verify_simple_encoded(
        SEGWIT_ADDRESS,
        "",
        "AkcwRAIgM2gBAQqvZX15ZiysmKmQpDrG83avLIT492QBzLnQIxYCIBaTpOaD20qRlEylyxFSeEA2ba9YOixpX8z46TSDtS40ASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI="
      ).is_ok()
    );

    assert!(
      verify::verify_simple_encoded(
        SEGWIT_ADDRESS,
        "fail",
        "AkcwRAIgM2gBAQqvZX15ZiysmKmQpDrG83avLIT492QBzLnQIxYCIBaTpOaD20qRlEylyxFSeEA2ba9YOixpX8z46TSDtS40ASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI="
      ).is_err()
    );

    assert!(
      verify::verify_simple_encoded(
        SEGWIT_ADDRESS,
        "",
        "AkgwRQIhAPkJ1Q4oYS0htvyuSFHLxRQpFAY56b70UvE7Dxazen0ZAiAtZfFz1S6T6I23MWI2lK/pcNTWncuyL8UL+oMdydVgzAEhAsfxIAMZZEKUPYWI4BruhAQjzFT8FSFSajuFwrDL1Yhy"
      ).is_ok()
    );
  }

  #[test]
  fn simple_sign_p2wpkh() {
    assert_eq!(
      sign::sign_simple_encoded(SEGWIT_ADDRESS, "Hello World", WIF_PRIVATE_KEY).unwrap(),
      "AkgwRQIhAOzyynlqt93lOKJr+wmmxIens//zPzl9tqIOua93wO6MAiBi5n5EyAcPScOjf1lAqIUIQtr3zKNeavYabHyR8eGhowEhAsfxIAMZZEKUPYWI4BruhAQjzFT8FSFSajuFwrDL1Yhy"
    );

    assert_eq!(
      sign::sign_simple_encoded(SEGWIT_ADDRESS, "", WIF_PRIVATE_KEY).unwrap(),
      "AkgwRQIhAPkJ1Q4oYS0htvyuSFHLxRQpFAY56b70UvE7Dxazen0ZAiAtZfFz1S6T6I23MWI2lK/pcNTWncuyL8UL+oMdydVgzAEhAsfxIAMZZEKUPYWI4BruhAQjzFT8FSFSajuFwrDL1Yhy"
    );
  }

  #[test]
  fn roundtrip_p2wpkh_simple() {
    assert!(verify::verify_simple_encoded(
      SEGWIT_ADDRESS,
      "Hello World",
      &sign::sign_simple_encoded(SEGWIT_ADDRESS, "Hello World", WIF_PRIVATE_KEY).unwrap()
    )
    .is_ok());
  }

  #[test]
  fn roundtrip_p2wpkh_full() {
    assert!(verify::verify_full_encoded(
      SEGWIT_ADDRESS,
      "Hello World",
      &sign::sign_full_encoded(SEGWIT_ADDRESS, "Hello World", WIF_PRIVATE_KEY).unwrap()
    )
    .is_ok());
  }

  #[test]
  fn simple_verify_and_falsify_p2sh_p2wpkh() {
    assert!(verify::verify_simple_encoded(
        NESTED_SEGWIT_ADDRESS,
        "Hello World",
        "AkgwRQIhAMd2wZSY3x0V9Kr/NClochoTXcgDaGl3OObOR17yx3QQAiBVWxqNSS+CKen7bmJTG6YfJjsggQ4Fa2RHKgBKrdQQ+gEhAxa5UDdQCHSQHfKQv14ybcYm1C9y6b12xAuukWzSnS+w"
      ).is_ok()
    );

    assert!(verify::verify_simple_encoded(
        NESTED_SEGWIT_ADDRESS,
        "Hello World - this should fail",
        "AkgwRQIhAMd2wZSY3x0V9Kr/NClochoTXcgDaGl3OObOR17yx3QQAiBVWxqNSS+CKen7bmJTG6YfJjsggQ4Fa2RHKgBKrdQQ+gEhAxa5UDdQCHSQHfKQv14ybcYm1C9y6b12xAuukWzSnS+w"
      ).is_err()
    );
  }

  #[test]
  fn simple_sign_p2sh_p2wpkh() {
    assert_eq!(
      sign::sign_simple_encoded(NESTED_SEGWIT_ADDRESS, "Hello World", NESTED_SEGWIT_WIF_PRIVATE_KEY).unwrap(),
      "AkgwRQIhAMd2wZSY3x0V9Kr/NClochoTXcgDaGl3OObOR17yx3QQAiBVWxqNSS+CKen7bmJTG6YfJjsggQ4Fa2RHKgBKrdQQ+gEhAxa5UDdQCHSQHfKQv14ybcYm1C9y6b12xAuukWzSnS+w"
    );
  }

  #[test]
  fn roundtrip_p2sh_p2wpkh_simple() {
    assert!(verify::verify_simple_encoded(
      NESTED_SEGWIT_ADDRESS,
      "Hello World",
      &sign::sign_simple_encoded(
        NESTED_SEGWIT_ADDRESS,
        "Hello World",
        NESTED_SEGWIT_WIF_PRIVATE_KEY
      )
      .unwrap()
    )
    .is_ok());
  }

  #[test]
  fn roundtrip_p2sh_p2wpkh_full() {
    assert!(verify::verify_full_encoded(
      NESTED_SEGWIT_ADDRESS,
      "Hello World",
      &sign::sign_full_encoded(
        NESTED_SEGWIT_ADDRESS,
        "Hello World",
        NESTED_SEGWIT_WIF_PRIVATE_KEY
      )
      .unwrap()
    )
    .is_ok());
  }

  #[test]
  fn adding_aux_randomness_roundtrips() {
    let address = Address::from_str(TAPROOT_ADDRESS).unwrap().assume_checked();
    let message = "Hello World with aux randomness";
    let to_spend = create_to_spend(&address, message).unwrap();
    let to_sign = create_to_sign(&to_spend, None).unwrap();
    let private_key = PrivateKey::from_wif(WIF_PRIVATE_KEY).unwrap();

    let mut aux_rand = [0u8; 32];
    rand::rng().fill_bytes(&mut aux_rand);

    let witness =
      create_message_signature_taproot(&to_spend, &to_sign, private_key, Some(aux_rand));

    assert!(verify_simple(&address, message, witness).is_ok());
  }
}
