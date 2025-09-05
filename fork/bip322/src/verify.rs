use super::*;

/// Verifies the BIP-322 simple from spec-compliant string encodings.
#[allow(clippy::result_large_err)]
pub fn verify_simple_encoded(address: &str, message: &str, signature: &str) -> Result<()> {
  let address = Address::from_str(address)
    .context(error::AddressParse { address })?
    .assume_checked();

  let mut cursor = bitcoin::io::Cursor::new(
    general_purpose::STANDARD
      .decode(signature)
      .context(error::SignatureDecode { signature })?,
  );

  let witness =
    Witness::consensus_decode_from_finite_reader(&mut cursor).context(error::WitnessMalformed)?;

  verify_simple(&address, message, witness)
}

/// Verifies the BIP-322 full from spec-compliant string encodings.
#[allow(clippy::result_large_err)]
pub fn verify_full_encoded(address: &str, message: &str, to_sign: &str) -> Result<()> {
  let address = Address::from_str(address)
    .context(error::AddressParse { address })?
    .assume_checked();

  let mut cursor = bitcoin::io::Cursor::new(general_purpose::STANDARD.decode(to_sign).context(
    error::TransactionBase64Decode {
      transaction: to_sign,
    },
  )?);

  let to_sign = Transaction::consensus_decode_from_finite_reader(&mut cursor).context(
    error::TransactionConsensusDecode {
      transaction: to_sign,
    },
  )?;

  verify_full(&address, message, to_sign)
}

/// Verifies the BIP-322 simple from proper Rust types.
#[allow(clippy::result_large_err)]
pub fn verify_simple(
  address: &Address,
  message: impl AsRef<[u8]>,
  signature: Witness,
) -> Result<()> {
  verify_full(
    address,
    &message,
    create_to_sign(&create_to_spend(address, &message)?, Some(signature))?
      .extract_tx()
      .context(error::TransactionExtract)?,
  )
}

/// Verifies the BIP-322 full from proper Rust types.
#[allow(clippy::result_large_err)]
pub fn verify_full(
  address: &Address,
  message: impl AsRef<[u8]>,
  to_sign: Transaction,
) -> Result<()> {
  match address.to_address_data() {
    AddressData::Segwit { witness_program }
      if witness_program.version().to_num() == 1 && witness_program.program().len() == 32 =>
    {
      let pub_key = XOnlyPublicKey::from_slice(witness_program.program().as_bytes())
        .map_err(|_| Error::InvalidPublicKey)?;

      verify_full_p2tr(address, message, to_sign, pub_key)
    }
    AddressData::Segwit { witness_program }
      if witness_program.version().to_num() == 0
        && witness_program.program().len() == 20
        && to_sign.input.len() > 0
        && to_sign.input[0].witness.len() > 1 =>
    {
      let pub_key =
        PublicKey::from_slice(&to_sign.input[0].witness[1]).map_err(|_| Error::InvalidPublicKey)?;

      verify_full_p2wpkh(address, message, to_sign, pub_key, false)
    }
    AddressData::P2sh { script_hash: _ }
      if to_sign.input.len() > 0 && to_sign.input[0].witness.len() > 1 =>
    {
      let pub_key =
        PublicKey::from_slice(&to_sign.input[0].witness[1]).map_err(|_| Error::InvalidPublicKey)?;

      verify_full_p2wpkh(address, message, to_sign, pub_key, true)
    }
    _ => Err(Error::UnsupportedAddress {
      address: address.to_string(),
    }),
  }
}

#[allow(clippy::result_large_err)]
fn verify_full_p2wpkh(
  address: &Address,
  message: impl AsRef<[u8]>,
  to_sign: Transaction,
  pub_key: PublicKey,
  is_p2sh: bool,
) -> Result<()> {
  let to_spend = create_to_spend(address, message)?;
  let to_sign = create_to_sign(&to_spend, Some(to_sign.input[0].witness.clone()))?;

  let to_spend_outpoint = OutPoint {
    txid: to_spend.compute_txid(),
    vout: 0,
  };

  if to_spend_outpoint != to_sign.unsigned_tx.input[0].previous_output {
    return Err(Error::ToSignInvalid);
  }

  let witness = if let Some(witness) = to_sign.inputs[0].final_script_witness.clone() {
    witness
  } else {
    return Err(Error::WitnessEmpty);
  };

  if witness.len() != 2 {
    return Err(Error::InvalidWitness);
  }

  let encoded_signature = witness.to_vec()[0].clone();
  let witness_pub_key = &witness.to_vec()[1];

  if &pub_key.to_bytes() != witness_pub_key {
    return Err(Error::PublicKeyMismatch);
  }

  let signature_length = encoded_signature.len();

  let (signature, sighash_type) = match signature_length {
    71 | 72 => (
      bitcoin::secp256k1::ecdsa::Signature::from_der(
        &encoded_signature.as_slice()[..signature_length - 1],
      )
      .context(error::SignatureInvalid)?,
      EcdsaSighashType::from_consensus(encoded_signature[signature_length - 1] as u32),
    ),
    _ => {
      return Err(Error::SignatureLength {
        length: encoded_signature.len(),
        encoded_signature,
      })
    }
  };

  if !(sighash_type == EcdsaSighashType::All) {
    return Err(Error::SigHashTypeUnsupported {
      sighash_type: sighash_type.to_string(),
    });
  }

  let mut sighash_cache = SighashCache::new(to_sign.unsigned_tx);

  let sighash = sighash_cache
    .p2wpkh_signature_hash(
      0,
      &if is_p2sh {
        ScriptBuf::new_p2wpkh(&pub_key.wpubkey_hash().unwrap())
      } else {
        to_spend.output[0].script_pubkey.clone()
      },
      to_spend.output[0].value,
      sighash_type,
    )
    .expect("signature hash should compute");

  let message =
    Message::from_digest_slice(sighash.as_ref()).expect("should be cryptographically secure hash");

  Secp256k1::verification_only()
    .verify_ecdsa(&message, &signature, &pub_key.inner)
    .context(error::SignatureInvalid)?;

  Ok(())
}

#[allow(clippy::result_large_err)]
fn verify_full_p2tr(
  address: &Address,
  message: impl AsRef<[u8]>,
  to_sign: Transaction,
  pub_key: XOnlyPublicKey,
) -> Result<()> {
  let to_spend = create_to_spend(address, message)?;
  let to_sign = create_to_sign(&to_spend, Some(to_sign.input[0].witness.clone()))?;

  let to_spend_outpoint = OutPoint {
    txid: to_spend.compute_txid(),
    vout: 0,
  };

  if to_spend_outpoint != to_sign.unsigned_tx.input[0].previous_output {
    return Err(Error::ToSignInvalid);
  }

  let witness = if let Some(witness) = to_sign.inputs[0].final_script_witness.clone() {
    witness
  } else {
    return Err(Error::WitnessEmpty);
  };

  let encoded_signature = witness.to_vec()[0].clone();

  let (signature, sighash_type) = match encoded_signature.len() {
    65 => (
      Signature::from_slice(&encoded_signature.as_slice()[..64])
        .context(error::SignatureInvalid)?,
      TapSighashType::from_consensus_u8(encoded_signature[64])
        .context(error::SigHashTypeInvalid)?,
    ),
    64 => (
      Signature::from_slice(encoded_signature.as_slice()).context(error::SignatureInvalid)?,
      TapSighashType::Default,
    ),
    _ => {
      return Err(Error::SignatureLength {
        length: encoded_signature.len(),
        encoded_signature,
      })
    }
  };

  if !(sighash_type == TapSighashType::All || sighash_type == TapSighashType::Default) {
    return Err(Error::SigHashTypeUnsupported {
      sighash_type: sighash_type.to_string(),
    });
  }

  let mut sighash_cache = SighashCache::new(to_sign.unsigned_tx);

  let sighash = sighash_cache
    .taproot_key_spend_signature_hash(
      0,
      &sighash::Prevouts::All(&[TxOut {
        value: Amount::from_sat(0),
        script_pubkey: to_spend.output[0].clone().script_pubkey,
      }]),
      sighash_type,
    )
    .expect("signature hash should compute");

  let message =
    Message::from_digest_slice(sighash.as_ref()).expect("should be cryptographically secure hash");

  Secp256k1::verification_only()
    .verify_schnorr(&signature, &message, &pub_key)
    .context(error::SignatureInvalid)
}
