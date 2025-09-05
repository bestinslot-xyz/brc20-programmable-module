use super::*;

/// Signs the BIP-322 simple from spec-compliant string encodings.
#[allow(clippy::result_large_err)]
pub fn sign_simple_encoded(address: &str, message: &str, wif_private_key: &str) -> Result<String> {
  let address = Address::from_str(address)
    .context(error::AddressParse { address })?
    .assume_checked();

  let private_key = PrivateKey::from_wif(wif_private_key).context(error::PrivateKeyParse)?;

  let witness = sign_simple(&address, message, private_key)?;

  let mut buffer = Vec::new();

  witness
    .consensus_encode(&mut buffer)
    .context(error::WitnessEncoding)?;

  Ok(general_purpose::STANDARD.encode(buffer))
}

/// Signs the BIP-322 full from spec-compliant string encodings.
#[allow(clippy::result_large_err)]
pub fn sign_full_encoded(address: &str, message: &str, wif_private_key: &str) -> Result<String> {
  let address = Address::from_str(address)
    .context(error::AddressParse { address })?
    .assume_checked();

  let private_key = PrivateKey::from_wif(wif_private_key).context(error::PrivateKeyParse)?;

  let tx = sign_full(&address, message, private_key)?;

  let mut buffer = Vec::new();

  tx.consensus_encode(&mut buffer)
    .context(error::TransactionEncode)?;

  Ok(general_purpose::STANDARD.encode(buffer))
}

/// Signs in the BIP-322 simple format from proper Rust types and returns the witness.
#[allow(clippy::result_large_err)]
pub fn sign_simple(
  address: &Address,
  message: impl AsRef<[u8]>,
  private_key: PrivateKey,
) -> Result<Witness> {
  Ok(
    sign_full(address, message, private_key)?.input[0]
      .witness
      .clone(),
  )
}

/// Signs in the BIP-322 full format from proper Rust types and returns the full transaction.
#[allow(clippy::result_large_err)]
pub fn sign_full(
  address: &Address,
  message: impl AsRef<[u8]>,
  private_key: PrivateKey,
) -> Result<Transaction> {
  let to_spend = create_to_spend(address, message)?;
  let mut to_sign = create_to_sign(&to_spend, None)?;

  let witness = match address.to_address_data() {
    AddressData::Segwit { witness_program } => {
      let version = witness_program.version().to_num();
      let program_len = witness_program.program().len();

      match version {
        0 => {
          if program_len != 20 {
            return Err(Error::NotKeyPathSpend);
          }
          create_message_signature_p2wpkh(&to_spend, &to_sign, private_key, false)
        }
        1 => {
          if program_len != 32 {
            return Err(Error::NotKeyPathSpend);
          }
          create_message_signature_taproot(&to_spend, &to_sign, private_key, None)
        }
        _ => {
          return Err(Error::UnsupportedAddress {
            address: address.to_string(),
          })
        }
      }
    }
    AddressData::P2sh { script_hash: _ } => {
      create_message_signature_p2wpkh(&to_spend, &to_sign, private_key, true)
    }
    _ => {
      return Err(Error::UnsupportedAddress {
        address: address.to_string(),
      });
    }
  };

  to_sign.inputs[0].final_script_witness = Some(witness);

  to_sign.extract_tx().context(error::TransactionExtract)
}

/// Sign for segwit inputs
pub fn create_message_signature_p2wpkh(
  to_spend_tx: &Transaction,
  to_sign: &Psbt,
  private_key: PrivateKey,
  is_p2sh: bool,
) -> Witness {
  let secp = Secp256k1::new();
  let sighash_type = EcdsaSighashType::All;
  let mut sighash_cache = SighashCache::new(to_sign.unsigned_tx.clone());

  let pub_key = private_key.public_key(&secp);

  let sighash = sighash_cache
    .p2wpkh_signature_hash(
      0,
      &if is_p2sh {
        ScriptBuf::new_p2wpkh(&pub_key.wpubkey_hash().unwrap())
      } else {
        to_spend_tx.output[0].script_pubkey.clone()
      },
      to_spend_tx.output[0].value,
      sighash_type,
    )
    .expect("signature hash should compute");

  let signature = secp.sign_ecdsa(
    &secp256k1::Message::from_digest_slice(sighash.as_ref())
      .expect("should be cryptographically secure hash"),
    &private_key.inner,
  );

  let witness = sighash_cache
    .witness_mut(0)
    .expect("getting mutable witness reference should work");

  witness.push(
    bitcoin::ecdsa::Signature {
      signature,
      sighash_type,
    }
    .to_vec(),
  );

  witness.push(pub_key.to_bytes());

  witness.to_owned()
}

/// Sign for taproot inputs
pub fn create_message_signature_taproot(
  to_spend_tx: &Transaction,
  to_sign: &Psbt,
  private_key: PrivateKey,
  aux_rand: Option<[u8; 32]>,
) -> Witness {
  let mut to_sign = to_sign.clone();

  let secp = Secp256k1::new();
  let key_pair = Keypair::from_secret_key(&secp, &private_key.inner);

  let (x_only_public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);
  to_sign.inputs[0].tap_internal_key = Some(x_only_public_key);

  let sighash_type = TapSighashType::All;

  let mut sighash_cache = SighashCache::new(to_sign.unsigned_tx.clone());

  let sighash = sighash_cache
    .taproot_key_spend_signature_hash(
      0,
      &sighash::Prevouts::All(&[TxOut {
        value: Amount::from_sat(0),
        script_pubkey: to_spend_tx.output[0].clone().script_pubkey,
      }]),
      sighash_type,
    )
    .expect("signature hash should compute");

  let key_pair = key_pair
    .tap_tweak(&secp, to_sign.inputs[0].tap_merkle_root)
    .to_keypair();

  let signature = if let Some(aux_rand) = aux_rand {
    secp.sign_schnorr_with_aux_rand(
      &secp256k1::Message::from_digest_slice(sighash.as_ref())
        .expect("should be cryptographically secure hash"),
      &key_pair,
      &aux_rand,
    )
  } else {
    secp.sign_schnorr_no_aux_rand(
      &secp256k1::Message::from_digest_slice(sighash.as_ref())
        .expect("should be cryptographically secure hash"),
      &key_pair,
    )
  };

  let witness = sighash_cache
    .witness_mut(0)
    .expect("getting mutable witness reference should work");

  witness.push(
    bitcoin::taproot::Signature {
      signature,
      sighash_type,
    }
    .to_vec(),
  );

  witness.to_owned()
}
