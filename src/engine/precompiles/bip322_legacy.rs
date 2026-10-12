//! BIP-322 simple signature verification with the rules of the `bip322` crate v0.0.10
//! (CC0-1.0, https://github.com/rust-bitcoin/bip322), which the BIP322 precompile applies
//! to blocks below the v0.0.11 activation height (see `use_bip322_v0_0_11`).
//!
//! The control flow mirrors `bip322::verify_simple` v0.0.10. Inputs that make v0.0.10 panic
//! (an empty P2TR witness, an uncompressed P2SH-P2WPKH key) return an error here.

use bitcoin::absolute::LockTime;
use bitcoin::address::AddressData;
use bitcoin::blockdata::script;
use bitcoin::hashes::{sha256, Hash, HashEngine};
use bitcoin::opcodes;
use bitcoin::psbt::Psbt;
use bitcoin::secp256k1::{ecdsa, schnorr, Message, Secp256k1, XOnlyPublicKey};
use bitcoin::sighash::{Prevouts, SighashCache, TapSighashType};
use bitcoin::transaction::Version;
use bitcoin::{
    Address, Amount, EcdsaSighashType, OutPoint, PublicKey, ScriptBuf, Sequence, Transaction, TxIn,
    TxOut, Witness,
};

const BIP322_TAG: &str = "BIP0322-signed-message";

pub fn verify_simple(address: &Address, message: &[u8], signature: Witness) -> Result<(), ()> {
    let to_spend = create_to_spend(address, message);
    let to_sign = create_to_sign(&to_spend, signature)?
        .extract_tx()
        .map_err(|_| ())?;
    verify_full(address, &to_spend, to_sign)
}

fn verify_full(address: &Address, to_spend: &Transaction, to_sign: Transaction) -> Result<(), ()> {
    match address.to_address_data() {
        AddressData::Segwit { witness_program }
            if witness_program.version().to_num() == 1 && witness_program.program().len() == 32 =>
        {
            let pub_key =
                XOnlyPublicKey::from_slice(witness_program.program().as_bytes()).map_err(|_| ())?;
            verify_full_p2tr(to_spend, to_sign, pub_key)
        }
        AddressData::Segwit { witness_program }
            if witness_program.version().to_num() == 0
                && witness_program.program().len() == 20
                && !to_sign.input.is_empty()
                && to_sign.input[0].witness.len() > 1 =>
        {
            let pub_key = PublicKey::from_slice(&to_sign.input[0].witness[1]).map_err(|_| ())?;
            verify_full_p2wpkh(to_spend, to_sign, pub_key, false)
        }
        AddressData::P2sh { script_hash: _ }
            if !to_sign.input.is_empty() && to_sign.input[0].witness.len() > 1 =>
        {
            let pub_key = PublicKey::from_slice(&to_sign.input[0].witness[1]).map_err(|_| ())?;
            verify_full_p2wpkh(to_spend, to_sign, pub_key, true)
        }
        _ => Err(()),
    }
}

fn verify_full_p2wpkh(
    to_spend: &Transaction,
    to_sign: Transaction,
    pub_key: PublicKey,
    is_p2sh: bool,
) -> Result<(), ()> {
    let to_sign = create_to_sign(to_spend, to_sign.input[0].witness.clone())?;

    let to_spend_outpoint = OutPoint {
        txid: to_spend.compute_txid(),
        vout: 0,
    };
    if to_spend_outpoint != to_sign.unsigned_tx.input[0].previous_output {
        return Err(());
    }

    let Some(witness) = to_sign.inputs[0].final_script_witness.clone() else {
        return Err(());
    };
    if witness.len() != 2 {
        return Err(());
    }

    let encoded_signature = witness.to_vec()[0].clone();
    let witness_pub_key = &witness.to_vec()[1];
    if &pub_key.to_bytes() != witness_pub_key {
        return Err(());
    }

    // v0.0.10 only accepts 71 and 72 byte encodings (DER signature plus sighash byte)
    let signature_length = encoded_signature.len();
    let (signature, sighash_type) = match signature_length {
        71 | 72 => (
            ecdsa::Signature::from_der(&encoded_signature[..signature_length - 1])
                .map_err(|_| ())?,
            EcdsaSighashType::from_consensus(encoded_signature[signature_length - 1] as u32),
        ),
        _ => return Err(()),
    };
    if sighash_type != EcdsaSighashType::All {
        return Err(());
    }

    let script_code = if is_p2sh {
        ScriptBuf::new_p2wpkh(&pub_key.wpubkey_hash().map_err(|_| ())?)
    } else {
        to_spend.output[0].script_pubkey.clone()
    };
    let sighash = SighashCache::new(to_sign.unsigned_tx)
        .p2wpkh_signature_hash(0, &script_code, to_spend.output[0].value, sighash_type)
        .map_err(|_| ())?;
    let message = Message::from_digest(sighash.to_byte_array());

    Secp256k1::verification_only()
        .verify_ecdsa(&message, &signature, &pub_key.inner)
        .map_err(|_| ())
}

fn verify_full_p2tr(
    to_spend: &Transaction,
    to_sign: Transaction,
    pub_key: XOnlyPublicKey,
) -> Result<(), ()> {
    let to_sign = create_to_sign(to_spend, to_sign.input[0].witness.clone())?;

    let to_spend_outpoint = OutPoint {
        txid: to_spend.compute_txid(),
        vout: 0,
    };
    if to_spend_outpoint != to_sign.unsigned_tx.input[0].previous_output {
        return Err(());
    }

    let Some(witness) = to_sign.inputs[0].final_script_witness.clone() else {
        return Err(());
    };
    let Some(encoded_signature) = witness.to_vec().first().cloned() else {
        return Err(());
    };

    let (signature, sighash_type) = match encoded_signature.len() {
        65 => (
            schnorr::Signature::from_slice(&encoded_signature[..64]).map_err(|_| ())?,
            TapSighashType::from_consensus_u8(encoded_signature[64]).map_err(|_| ())?,
        ),
        64 => (
            schnorr::Signature::from_slice(&encoded_signature).map_err(|_| ())?,
            TapSighashType::Default,
        ),
        _ => return Err(()),
    };
    if !(sighash_type == TapSighashType::All || sighash_type == TapSighashType::Default) {
        return Err(());
    }

    let sighash = SighashCache::new(to_sign.unsigned_tx)
        .taproot_key_spend_signature_hash(
            0,
            &Prevouts::All(&[TxOut {
                value: Amount::from_sat(0),
                script_pubkey: to_spend.output[0].script_pubkey.clone(),
            }]),
            sighash_type,
        )
        .map_err(|_| ())?;
    let message = Message::from_digest(sighash.to_byte_array());

    Secp256k1::verification_only()
        .verify_schnorr(&signature, &message, &pub_key)
        .map_err(|_| ())
}

fn tagged_hash(tag: &str, message: &[u8]) -> [u8; 32] {
    let tag_hash = sha256::Hash::hash(tag.as_bytes());
    let mut engine = sha256::Hash::engine();
    engine.input(tag_hash.as_byte_array());
    engine.input(tag_hash.as_byte_array());
    engine.input(message);
    sha256::Hash::from_engine(engine).to_byte_array()
}

fn create_to_spend(address: &Address, message: &[u8]) -> Transaction {
    Transaction {
        version: Version(0),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: Hash::all_zeros(),
                vout: 0xFFFFFFFF,
            },
            script_sig: script::Builder::new()
                .push_int(0)
                .push_slice(tagged_hash(BIP322_TAG, message))
                .into_script(),
            sequence: Sequence(0),
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(0),
            script_pubkey: address.script_pubkey(),
        }],
    }
}

fn create_to_sign(to_spend: &Transaction, witness: Witness) -> Result<Psbt, ()> {
    let to_sign = Transaction {
        version: Version(0),
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: to_spend.compute_txid(),
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0),
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(0),
            script_pubkey: script::Builder::new()
                .push_opcode(opcodes::all::OP_RETURN)
                .into_script(),
        }],
    };

    let mut psbt = Psbt::from_unsigned_tx(to_sign).map_err(|_| ())?;
    psbt.inputs[0].witness_utxo = Some(TxOut {
        value: Amount::from_sat(0),
        script_pubkey: to_spend.output[0].script_pubkey.clone(),
    });
    psbt.inputs[0].final_script_witness = Some(witness);
    Ok(psbt)
}
