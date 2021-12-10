use std::{
	fs::File,
	io::{BufReader, Read, Write, Result},
	path::PathBuf,
	sync::Arc,
};

use base64::{encode, decode};
use ethcore_accounts::{AccountProvider, AccountProviderSettings};
use ethereum_types::{H160, H256};
use ethkey::Password;
use ethstore::accounts_dir::RootDiskDirectory;
use jsonrpc_core::IoHandler;
use serde_json::Value;

use crate::{
    metadata::Metadata,
    secretstore::{SecretStore, SecretStoreClient},
    util::{password_prompt, accounts_prompt},
};

pub struct Dependencies {
	pub address: H160,
	pub accounts: Arc<AccountProvider>,
	pub password: Password,
}

impl Dependencies {
	pub fn new() -> Self {
		let dir = RootDiskDirectory::create("data").unwrap();
		let store = ethstore::EthStore::open_with_iterations(Box::new(dir), 1024).unwrap();
		let ap = AccountProvider::new(Box::new(store), AccountProviderSettings::default());
		let accounts = ap.accounts().unwrap();
		let mut address = H160::zero();
		let mut password = Password::from("".to_string());
		match accounts.len() {
			0 => {
				password = password_prompt().unwrap();
				address = ap.new_account(&password).unwrap();
			},
			1 => {
				assert_eq!(ap.default_account().unwrap(), accounts[0]);
				// password check
				password = password_prompt().unwrap();
				if ap.test_password(&accounts[0], &password).unwrap() {
					address = accounts[0];
				} else {
					println!("Password does not match");
				}
			},
			_ => {
				// TODO: account selection
				let num = accounts_prompt(accounts);
				println!("num is : {:?}", num);
			}
		}
		Dependencies {
			address: address,
			accounts: Arc::new(ap),
			password: password,
		}
	}

	pub fn transient() -> Self {
		let ap = AccountProvider::transient_provider();
		let password = Password::from("");
		let address = ap.new_account(&password).unwrap();
		Dependencies {
			address: address,
			accounts: Arc::new(ap),
			password: password,
		}
	}

	// TODO: refactor
	pub fn default() -> Self {
		let dir = RootDiskDirectory::create("data").unwrap();
		let store = ethstore::EthStore::open_with_iterations(Box::new(dir), 1024).unwrap();
		let ap = AccountProvider::new(Box::new(store), AccountProviderSettings::default());
		let address = ap.default_account().unwrap();
		let password = Password::from("".to_string());
		Dependencies {
			address: address,
			accounts: Arc::new(ap),
			password: password,
		}
	}

	fn client(&self) -> SecretStoreClient {
		SecretStoreClient::new(&self.accounts)
	}

	pub fn default_client(&self) -> IoHandler<Metadata> {
		let mut io = IoHandler::default();
		io.extend_with(self.client().to_delegate());
		io
	}

	pub fn encode_file(&self, file: PathBuf) -> String {
		let f = File::open(file).expect("could not open file");
		let mut file = BufReader::new(f);
		let mut buf = vec![];
		file.read_to_end(&mut buf).expect("could not read file");
		encode(&buf)
	}

	pub fn decode_reader(&self, string: &str) -> Result<Vec<u8>> {
		let decoded = decode(string).unwrap();
		Ok(decoded)
	}

	pub fn post_to_ipfs(&self, dockey_id: H256, encrypted_secret_document: &str) -> Result<Value> {
		let filename = &hex::encode(H256::as_bytes(&dockey_id));
		let mut f = File::create(&filename).unwrap();
		f.write_all(encrypted_secret_document.as_bytes()).unwrap();

		let client = reqwest::Client::new();
		let form = reqwest::multipart::Form::new().file("file", &filename).unwrap();
		let mut res = client.post("https://ipfs.infura.io:5001/api/v0/add")
						.multipart(form)
						.send().unwrap();
		let mut buf: Vec<u8> = vec![];
		res.copy_to(&mut buf).unwrap();
		let response = String::from_utf8(buf.to_vec()).unwrap();
		let v: Value = serde_json::from_str(&response).unwrap();
		// TODO: ipfs_hash to file
		Ok(v)
	}

	pub fn cat_ipfs(&self, ipfs_uri: &str) -> Result<String> {
		let param = "https://ipfs.infura.io:5001/api/v0/cat?arg=".to_owned() + &ipfs_uri.to_owned();
		let mut buf: Vec<u8> = vec![];
		let mut res = reqwest::get(&param).unwrap();
		res.copy_to(&mut buf).unwrap();
		Ok(String::from_utf8(buf.to_vec()).unwrap())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::document_key::EncryptedDocumentKey;
	use ethkey::{KeyPair, Signature, verify_public};
	use jsonrpc_core::Success;
	use parity_crypto::DEFAULT_MAC;

	#[test]
	fn rpc_encrypt_and_decrypt() {
		let deps = Dependencies::transient();
		let io = deps.default_client();

		let secret = "c1f1cfe279a5c350d13795bce162941967340c8a228e6ba175489afc564a5bef".parse().unwrap();
		deps.accounts.insert_account(secret, &"password".into()).unwrap();

		let encryption_request = r#"{"jsonrpc": "2.0", "method": "secretstore_encrypt", "params":[
		"0x5c2f3b4ec0c2234f8358697edc8b82a62e3ac995", "password",
		"0x0440262acc06f1e13cb11b34e792cdf698673a16bb812163cb52689ac34c94ae47047b58f58d8b596d21ac7b03a55896132d07a7dc028b2dad88f6c5a90623fa5b30ff4b1ba385a98c970432d13417cf6d7facd62f86faaef15ca993735890da0cb3e417e2740fc72de7501eef083a12dd5a9ebe513b592b1740848576a936a1eb88fc553fc624b1cae41a0a4e074e34e2aaae686709f08d70e505c5acba12ef96017e89be675a2adb07c72c4e95814fbf",
		"0xdeadbeef"
		], "id": 1}"#;
		let encryption_response = io.handle_request_sync(encryption_request).unwrap();
		let encryption_response: Success = serde_json::from_str(&encryption_response).unwrap();

		let decryption_request_left = r#"{"jsonrpc": "2.0", "method": "secretstore_decrypt", "params":[
			"0x5c2f3b4ec0c2234f8358697edc8b82a62e3ac995", "password",
			"0x0440262acc06f1e13cb11b34e792cdf698673a16bb812163cb52689ac34c94ae47047b58f58d8b596d21ac7b03a55896132d07a7dc028b2dad88f6c5a90623fa5b30ff4b1ba385a98c970432d13417cf6d7facd62f86faaef15ca993735890da0cb3e417e2740fc72de7501eef083a12dd5a9ebe513b592b1740848576a936a1eb88fc553fc624b1cae41a0a4e074e34e2aaae686709f08d70e505c5acba12ef96017e89be675a2adb07c72c4e95814fbf",""#;
		let decryption_request_mid = encryption_response.result.as_str().unwrap();
		let decryption_request_right = r#""
			], "id": 2}"#;
		let decryption_request = decryption_request_left.to_owned() + decryption_request_mid + decryption_request_right;
		let decryption_response = io.handle_request_sync(&decryption_request).unwrap();
		assert_eq!(decryption_response, r#"{"jsonrpc":"2.0","result":"0xdeadbeef","id":2}"#);
	}

	#[test]
	fn rpc_shadow_decrypt() {
		let deps = Dependencies::transient();
		let io = deps.default_client();

		// insert new account
		let secret = "82758356bf46b42710d3946a8efa612b7bf5e125e4d49f28facf1139db4a46f4".parse().unwrap();
		deps.accounts.insert_account(secret, &"password".into()).unwrap();

		// execute decryption request
		let decryption_request = r#"{"jsonrpc": "2.0", "method": "secretstore_shadowDecrypt", "params":[
			"0x00dfE63B22312ab4329aD0d28CaD8Af987A01932", "password",
			"0x843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91",
			"0x07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3",
			["0x049ce50bbadb6352574f2c59742f78df83333975cbd5cbb151c6e8628749a33dc1fa93bb6dffae5994e3eb98ae859ed55ee82937538e6adb054d780d1e89ff140f121529eeadb1161562af9d3342db0008919ca280a064305e5a4e518e93279de7a9396fe5136a9658e337e8e276221248c381c5384cd1ad28e5921f46ff058d5fbcf8a388fc881d0dd29421c218d51761"],
			"0x2ddec1f96229efa2916988d8b2a82a47ef36f71c"
		], "id": 1}"#;
		let decryption_response = io.handle_request_sync(&decryption_request).unwrap();
		assert_eq!(decryption_response, r#"{"jsonrpc":"2.0","result":"0xdeadbeef","id":1}"#);
	}

	#[test]
	fn rpc_sign_raw_hash() {
		let deps = Dependencies::transient();
		let io = deps.default_client();

		// insert new account
		let secret = "82758356bf46b42710d3946a8efa612b7bf5e125e4d49f28facf1139db4a46f4".parse().unwrap();
		let key_pair = KeyPair::from_secret(secret).unwrap();
		deps.accounts.insert_account(key_pair.secret().clone(), &"password".into()).unwrap();

		// execute signing request
		let signing_request = r#"{"jsonrpc": "2.0", "method": "secretstore_signRawHash", "params":[
			"0x00dfE63B22312ab4329aD0d28CaD8Af987A01932", "password", "0x0000000000000000000000000000000000000000000000000000000000000001"
		], "id": 1}"#;
		let signing_response = io.handle_request_sync(&signing_request).unwrap();
		let signing_response = signing_response.replace(r#"{"jsonrpc":"2.0","result":"0x"#, "");
		let signing_response = signing_response.replace(r#"","id":1}"#, "");
		let signature: Signature = signing_response.parse().unwrap();

		let hash = "0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap();
		assert!(verify_public(key_pair.public(), &signature, &hash).unwrap());
	}

	#[test]
	fn rpc_generate_document_key() {
		let deps = Dependencies::transient();
		let io = deps.default_client();

		// insert new account
		let secret = "82758356bf46b42710d3946a8efa612b7bf5e125e4d49f28facf1139db4a46f4".parse().unwrap();
		let key_pair = KeyPair::from_secret(secret).unwrap();
		deps.accounts.insert_account(key_pair.secret().clone(), &"password".into()).unwrap();

		// execute generation request
		let generation_request = r#"{"jsonrpc": "2.0", "method": "secretstore_generateDocumentKey", "params":[
			"0x00dfE63B22312ab4329aD0d28CaD8Af987A01932", "password",
			"0x843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91"
		], "id": 1}"#;
		let generation_response = io.handle_request_sync(&generation_request).unwrap();
		let generation_response = generation_response.replace(r#"{"jsonrpc":"2.0","result":"#, "");
		let generation_response = generation_response.replace(r#","id":1}"#, "");
		let generation_response: EncryptedDocumentKey = serde_json::from_str(&generation_response).unwrap();

		// the only thing we can check is that 'encrypted_key' can be decrypted by passed account
		assert!(deps.accounts.decrypt(
			"00dfE63B22312ab4329aD0d28CaD8Af987A01932".parse().unwrap(),
			Some("password".into()),
			&DEFAULT_MAC,
			&generation_response.encrypted_key.0).is_ok());
		}
}
