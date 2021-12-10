use std::collections::BTreeSet;
use crate::document_key::EncryptedDocumentKey;
use crate::errors;
use ethereum_types::{H256, H512};
use ethkey::{self, Public, Generator, Random, math, Secret};
use jsonrpc_core::Error;
use parity_bytes::Bytes;
use parity_crypto as crypto;
use rand::RngCore;
use rand::rngs::OsRng;
use tiny_keccak::Keccak;

const INIT_VEC_LEN: usize = 16;

pub fn generate_document_key(account_public: Public, server_key_public: Public) -> Result<EncryptedDocumentKey, Error> {
	let document_key = Random.generate().map_err(errors::encryption)?;
	let (common_point, encrypted_point) = encrypt_secret(document_key.public(), &server_key_public)?;
	let encrypted_key = ethkey::crypto::ecies::encrypt(&account_public, &crypto::DEFAULT_MAC, document_key.public().as_bytes()).map_err(errors::encryption)?;

	Ok(EncryptedDocumentKey {
		common_point: common_point.into(),
		encrypted_point: encrypted_point.into(),
		encrypted_key: encrypted_key.into(),
	})
}

pub fn encrypt_document(key: Bytes, document: Bytes) -> Result<Bytes, Error> {
	let key = into_document_key(key)?;
	let iv = initialization_vector();
	let mut encrypted_document = vec![0; document.len() + iv.len()];
	{
		let (mut encryption_buffer, iv_buffer) = encrypted_document.split_at_mut(document.len());
		crypto::aes::encrypt_128_ctr(&key, &iv, &document, &mut encryption_buffer).map_err(errors::encryption)?;
		iv_buffer.copy_from_slice(&iv);
	}
	Ok(encrypted_document)
}

pub fn decrypt_document(key: Bytes, mut encrypted_document: Bytes) -> Result<Bytes, Error> {
	let encrypted_document_len = encrypted_document.len();
	if encrypted_document_len < INIT_VEC_LEN {
		return Err(errors::invalid_params("encryted_document", "invalied encrypted data"));
	}
	let key = into_document_key(key)?;
	let iv = encrypted_document.split_off(encrypted_document_len - INIT_VEC_LEN);
	let mut document = vec![0; encrypted_document_len - INIT_VEC_LEN];
	crypto::aes::decrypt_128_ctr(&key, &iv, &encrypted_document, &mut document).map_err(errors::encryption)?;

	Ok(document)
}

pub fn decrypt_document_with_shadow(decrypted_secret: Public, common_point: Public, shadows: Vec<Secret>, encrypted_document: Bytes) -> Result<Bytes, Error> {
	let key = decrypt_with_shadow_coefficients(decrypted_secret, common_point, shadows)?;
	decrypt_document(key.as_bytes().to_vec(), encrypted_document)
}

pub fn decrypt_with_shadow_coefficients(mut decrypted_shadow: Public, mut common_shadow_point: Public, shadow_coefficients: Vec<Secret>) -> Result<Public, Error> {
	let mut shadow_coefficients_sum = shadow_coefficients[0].clone();
	for shadow_coefficients in shadow_coefficients.iter().skip(1) {
		shadow_coefficients_sum.add(shadow_coefficients).map_err(errors::encryption)?;
	}
	math::public_mul_secret(&mut common_shadow_point, &shadow_coefficients_sum).map_err(errors::encryption)?;
	math::public_add(&mut decrypted_shadow, &common_shadow_point).map_err(errors::encryption)?;
	Ok(decrypted_shadow)
}

pub fn ordered_servers_keccak(servers_set: BTreeSet<H512>) -> H256 {
	let mut servers_set_keccak = Keccak::new_keccak256();
	for server in servers_set {
		servers_set_keccak.update(&server.0);
	}
	let mut servers_set_keccak_value = [0u8; 32];
	servers_set_keccak.finalize(&mut servers_set_keccak_value);
	servers_set_keccak_value.into()
}

fn into_document_key(key: Bytes) -> Result<Bytes, Error> {
	if key.len() != 64 {
		return Err(errors::invalid_params("key", "invalid public key length"));
	}
	Ok(key[..INIT_VEC_LEN].into())
}

fn initialization_vector() -> [u8; INIT_VEC_LEN] {
	let mut result = [0u8; INIT_VEC_LEN];
	OsRng.fill_bytes(&mut result);
	result
}

fn encrypt_secret(secret: &Public, joint_public: &Public) -> Result<(Public, Public), Error> {
	let key_pair = Random.generate().map_err(errors::encryption)?;
	let mut common_point = math::generation_point();
	math::public_mul_secret(&mut common_point, key_pair.secret()).map_err(errors::encryption)?;
	let mut encrypted_point = joint_public.clone();
	math::public_mul_secret(&mut encrypted_point, key_pair.secret()).map_err(errors::encryption)?;
	math::public_add(&mut encrypted_point, secret).map_err(errors::encryption)?;

	Ok((common_point, encrypted_point))
}

#[cfg(test)]
mod tests {
	use rustc_hex::FromHex;
	use parity_bytes::Bytes;
	use super::{encrypt_document, decrypt_document, decrypt_document_with_shadow};

	#[test]
	fn encrypt_and_decrypt_document() {
		let document_key: Bytes = "cac6c205eb06c8308d65156ff6c862c62b000b8ead121a4455a8ddeff7248128d895692136f240d5d1614dc7cc4147b1bd584bd617e30560bb872064d09ea325".from_hex().unwrap();
		let document: Bytes = b"Hello, world!"[..].into();
		let encrypted_document = encrypt_document(document_key.clone(), document.clone()).unwrap();
		assert!(document != encrypted_document);

		let decrypted_document = decrypt_document(document_key.clone(), encrypted_document).unwrap();
		assert_eq!(document, decrypted_document);
	}

	#[test]
	fn encrypt_and_shadow_decrypt() {
		let document: Bytes = "deadbeef".from_hex().unwrap();
		let decrypted_secret = "843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91".parse().unwrap();
		let common_point = "07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3".parse().unwrap();
		let shadows = vec!["46f542416216f66a7d7881f5a283d2a1ab7a87b381cbc5f29d0b093c7c89ee31".parse().unwrap()];
		let encrypted_document = "2ddec1f96229efa2916988d8b2a82a47ef36f71c".from_hex().unwrap();
		let decrypted_document = decrypt_document_with_shadow(decrypted_secret, common_point, shadows, encrypted_document).unwrap();
		assert_eq!(document, decrypted_document);
	}
}