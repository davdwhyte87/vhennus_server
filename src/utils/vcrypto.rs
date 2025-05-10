use k256::{FieldBytes, SecretKey};
use k256::ecdsa::{Signature, SigningKey};
use k256::ecdsa::signature::DigestSigner;
use sha2::{Digest, Sha256};
use sha2::digest::generic_array::GenericArray;
use crate::models::app_error::AppError;
use crate::models::request_models::BTransfer;

pub fn get_transaction_hash(tx:BTransfer) ->String{
    let mut hasher = Sha256::new();

    // Convert everything to bytes and hash
    hasher.update(tx.timestamp.to_be_bytes());
    hasher.update(tx.sender.as_bytes());
    hasher.update(tx.receiver.as_bytes());
    hasher.update(tx.amount.normalized().to_string().as_bytes());

    // Finalize and convert to hex string
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn sign_transaction(
    sender: &str,
    receiver: &str,
    amount: &str,
    timestamp: u64,
    id: &str,
    private_key_hex: String,
) -> Result<String, AppError> {

    let tx_data = format!("{sender}{receiver}{amount}{timestamp}{id}");
    log::debug!("🔏 tx_data to sign: {}", tx_data);
    log::debug!("raw hex: {}", hex::encode(tx_data.as_bytes()));


    // decode hex → Vec<u8>
    let priv_vec = hex::decode(private_key_hex)
        .map_err(|e| AppError::SignTransactionError)?;

    // try to turn it into exactly 32 bytes
    let priv_bytes: FieldBytes = GenericArray::clone_from_slice(&priv_vec);


    let signing_key = SigningKey::from_bytes(&priv_bytes)
        .map_err(|e| AppError::SignTransactionError)?;


    let signature: Signature = signing_key
        .sign_digest(Sha256::new().chain_update(tx_data.as_bytes()));
    let sig_bytes = signature.to_bytes();
    Ok(hex::encode(sig_bytes.as_slice()))
}

pub fn generate_compressed_pubkey(seed: &str) -> (String, String) {
    let hash = Sha256::digest(seed.as_bytes());
    let signing_key = SigningKey::from_bytes(&hash).unwrap();
    let verify_key = signing_key.verifying_key();
    let priv_bytes: FieldBytes = signing_key.to_bytes();
    let priv_hex = hex::encode(priv_bytes.as_slice());
    (priv_hex ,hex::encode(verify_key.to_encoded_point(true).as_bytes()) )
}