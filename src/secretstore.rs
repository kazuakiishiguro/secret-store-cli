use crate::bytes::Bytes;
use crate::document_key::EncryptedDocumentKey;
use crate::errors;
use crate::helpers::{
    decrypt_document, decrypt_document_with_shadow, encrypt_document, generate_document_key,
    ordered_servers_keccak,
};
use ethcore_accounts::AccountProvider;
use ethereum_types::{H160, H256, H512};
use ethkey::{Password, Secret};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use parity_crypto::DEFAULT_MAC;
use std::collections::BTreeSet;
use std::sync::Arc;

pub struct SecretStoreClient {
    accounts: Arc<AccountProvider>,
}

impl SecretStoreClient {
    // new
    pub fn new(store: &Arc<AccountProvider>) -> Self {
        SecretStoreClient {
            accounts: store.clone(),
        }
    }
    // decrypt_key
    fn decrypt_key(&self, address: H160, password: Password, key: Bytes) -> Result<Vec<u8>> {
        self.accounts
            .decrypt(address.into(), Some(password), &DEFAULT_MAC, &key.0)
            .map_err(|e| errors::account("Could not decrypt key.", e))
    }

    // decrypt_secret
    fn decrypt_secret(&self, address: H160, password: Password, key: Bytes) -> Result<Secret> {
        self.decrypt_key(address, password, key).and_then(|s| {
            Secret::from_unsafe_slice(&s).map_err(|e| errors::account("invalid secret", e))
        })
    }
}

#[rpc]
pub trait SecretStore {
    #[rpc(name = "secretstore_generateDocumentKey")]
    fn generate_document_key(
        &self,
        address: H160,
        password: Password,
        server_key_public: H512,
    ) -> Result<EncryptedDocumentKey>;

    #[rpc(name = "secretstore_encrypt")]
    fn encrypt(&self, address: H160, password: Password, key: Bytes, data: Bytes) -> Result<Bytes>;

    #[rpc(name = "secretstore_decrypt")]
    fn decrypt(&self, address: H160, password: Password, key: Bytes, data: Bytes) -> Result<Bytes>;

    #[rpc(name = "secretstore_shadowDecrypt")]
    fn shadow_decrypt(
        &self,
        address: H160,
        password: Password,
        decrypt_secret: H512,
        common_point: H512,
        decrypt_shadows: Vec<Bytes>,
        data: Bytes,
    ) -> Result<Bytes>;

    #[rpc(name = "secretstore_serversSetHash")]
    fn server_set_hash(&self, servers_set: BTreeSet<H512>) -> Result<H256>;

    #[rpc(name = "secretstore_signRawHash")]
    fn sign_raw_hash(&self, address: H160, password: Password, raw_hash: H256) -> Result<Bytes>;
}

impl SecretStore for SecretStoreClient {
    fn generate_document_key(
        &self,
        address: H160,
        password: Password,
        server_key_public: H512,
    ) -> Result<EncryptedDocumentKey> {
        let account_public = self
            .accounts
            .account_public(address.into(), &password)
            .map_err(|e| errors::account("Could not read account public.", e))?;
        generate_document_key(account_public, server_key_public.into())
    }

    fn encrypt(&self, address: H160, password: Password, key: Bytes, data: Bytes) -> Result<Bytes> {
        encrypt_document(self.decrypt_key(address, password, key)?, data.0).map(Into::into)
    }

    fn decrypt(&self, address: H160, password: Password, key: Bytes, data: Bytes) -> Result<Bytes> {
        decrypt_document(self.decrypt_key(address, password, key)?, data.0).map(Into::into)
    }

    fn shadow_decrypt(
        &self,
        address: H160,
        password: Password,
        decrypt_secret: H512,
        common_point: H512,
        decrypt_shadows: Vec<Bytes>,
        data: Bytes,
    ) -> Result<Bytes> {
        let mut shadows = Vec::with_capacity(decrypt_shadows.len());
        for decrypt_shadow in decrypt_shadows {
            shadows.push(self.decrypt_secret(address.clone(), password.clone(), decrypt_shadow)?);
        }

        decrypt_document_with_shadow(decrypt_secret.into(), common_point.into(), shadows, data.0)
            .map(Into::into)
    }

    fn server_set_hash(&self, servers_set: BTreeSet<H512>) -> Result<H256> {
        Ok(ordered_servers_keccak(servers_set))
    }

    fn sign_raw_hash(&self, address: H160, password: Password, raw_hash: H256) -> Result<Bytes> {
        self.accounts
            .sign(address.into(), Some(password), raw_hash.into())
            .map(|s| Bytes::new((*s).to_vec()))
            .map_err(|e| errors::account("Could not sign raw hash.", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethcore_accounts::AccountProvider;
    use ethkey::{verify_public, KeyPair, Signature};
    use parity_crypto::DEFAULT_MAC;

    #[test]
    fn ss_new_account_is_ok() {
        let ap = AccountProvider::transient_provider();
        let accounts = ap.new_account(&"test".into()).unwrap();
        let secretstore_new = SecretStoreClient::new(&Arc::new(ap));
        assert_eq!(accounts, secretstore_new.accounts.accounts().unwrap()[0]);
    }

    #[test]
    fn sign_raw_hash_is_ok() {
        let secret = "82758356bf46b42710d3946a8efa612b7bf5e125e4d49f28facf1139db4a46f4"
            .parse()
            .unwrap();
        let keypair = KeyPair::from_secret(secret).unwrap();

        let ap = AccountProvider::transient_provider();
        let account = ap
            .insert_account(keypair.secret().clone(), &"test".into())
            .unwrap();
        let secretstore_new = SecretStoreClient::new(&Arc::new(ap));
        let message = H256::random();
        let signing_response =
            secretstore_new.sign_raw_hash(account, "test".into(), message.clone());
        let signing_response = serde_json::to_string(&signing_response.unwrap()).unwrap();
        let signing_response = signing_response.replace("\"0x", "");
        let signing_response = signing_response.replace("\"", "");
        let signature: Signature = signing_response.parse().unwrap();
        assert!(verify_public(&keypair.public(), &signature, &message).unwrap());
    }

    #[test]
    fn generate_document_key_is_ok() {
        let secret = "82758356bf46b42710d3946a8efa612b7bf5e125e4d49f28facf1139db4a46f4"
            .parse()
            .unwrap();
        let keypair = KeyPair::from_secret(secret).unwrap();

        let ap = AccountProvider::transient_provider();
        let account = ap
            .insert_account(keypair.secret().clone(), &"test".into())
            .unwrap();
        let secretstore_new = SecretStoreClient::new(&Arc::new(ap));
        let skp = "843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91".as_bytes();
        let generation_response: EncryptedDocumentKey = secretstore_new
            .generate_document_key(
                account,
                "test".into(),
                H512::from_slice(&hex::decode(&skp).expect("decode failed")),
            )
            .unwrap();
        assert!(secretstore_new
            .accounts
            .decrypt(
                account,
                Some("test".into()),
                &DEFAULT_MAC,
                &generation_response.encrypted_key.0
            )
            .is_ok());
    }
}
