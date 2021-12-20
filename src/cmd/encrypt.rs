use std::{io::Result, path::PathBuf};

use ethereum_types::{H256, H512};
use serde_json::json;
use url::Url;

use crate::{dependency::Dependencies, document_key::EncryptedDocumentKey, util::create_dockey_id};

fn generate_server_key(dockey_id: H256, signed_dockey_id: &str) -> Result<String> {
    let base = Url::parse("http://localhost:8000/shadow/").unwrap();
    let path = format!(
        "{}{}{}{}",
        &hex::encode(H256::as_bytes(&dockey_id)),
        "/",
        signed_dockey_id,
        "/1?apikey=fa05a2e1-d323-4723-96b7-4d2695a61d3f"
    ); // TODO: work with threshhold
    let url = base.join(&path).unwrap();
    let client = reqwest::Client::new();
    let mut buf: Vec<u8> = vec![];
    let mut res = client.post(url.as_str()).send().unwrap();
    res.copy_to(&mut buf).unwrap();
    Ok(String::from_utf8(buf.to_vec()).unwrap())
}

fn store_document_key(
    dockey_id: H256,
    signed_dockey_id: &str,
    generation_response: EncryptedDocumentKey,
) -> Result<String> {
    let base = Url::parse("http://localhost:8000/shadow/").unwrap();
    let path = format!(
        "{}{}{}{}{}{}{}{}",
        &hex::encode(H256::as_bytes(&dockey_id)),
        "/",
        signed_dockey_id,
        "/",
        &hex::encode(H512::as_bytes(&generation_response.common_point)),
        "/",
        &hex::encode(H512::as_bytes(&generation_response.encrypted_point)),
        "?apikey=fa05a2e1-d323-4723-96b7-4d2695a61d3f"
    );
    let url = base.join(&path).unwrap();
    let client = reqwest::Client::new();
    let mut buf: Vec<u8> = vec![];
    let mut res = client.post(url.as_str()).send().unwrap();
    res.copy_to(&mut buf).unwrap();
    Ok(String::from_utf8(buf.to_vec()).unwrap())
}

pub fn encrypt(file: PathBuf) {
    let dockey_id = create_dockey_id(file.clone()).unwrap();
    let deps = Dependencies::transient();
    let io = deps.default_client();

    let params = json!({
        "jsonrpc": "2.0",
        "method": "secretstore_signRawHash",
        "params": [
            &deps.address,
            &deps.password,
            &dockey_id,
        ],
        "id": 1
    });
    let signed_dockey_id = io.handle_request_sync(&params.to_string()).unwrap();
    let signed_dockey_id = signed_dockey_id.replace(r#"{"jsonrpc":"2.0","result":"0x"#, "");
    let signed_dockey_id = signed_dockey_id.replace(r#"","id":1}"#, "");
    let server_key = generate_server_key(dockey_id, &signed_dockey_id).unwrap();
    let server_key = server_key.replace("\"", "");
    let params = json!({
        "jsonrpc": "2.0",
        "method": "secretstore_generateDocumentKey",
        "params": [
            &deps.address,
            &deps.password,
            server_key,
        ],
        "id": 1
    });
    let generation_response = io.handle_request_sync(&params.to_string()).unwrap();
    let generation_response = generation_response.replace(r#"{"jsonrpc":"2.0","result":"#, "");
    let generation_response = generation_response.replace(r#","id":1}"#, "");
    let generation_response: EncryptedDocumentKey =
        serde_json::from_str(&generation_response).unwrap();
    let encoded_file = deps.encode_file(file.clone());
    let params = json!({
        "jsonrpc": "2.0",
        "method": "secretstore_encrypt",
        "params": [
            &deps.address,
            &deps.password,
            &generation_response.encrypted_key,
            "0x".to_owned() + &hex::encode(encoded_file).to_string(),
        ],
        "id": 1
    });
    let encryption_response = io.handle_request_sync(&params.to_string()).unwrap();
    let encryption_response = encryption_response.replace(r#"{"jsonrpc":"2.0","result":"#, "");
    let encryption_response = encryption_response.replace(r#","id":1}"#, "");
    let result = deps.post_to_ipfs(dockey_id, &encryption_response).unwrap();
    println!("dockey_id: {:?}", dockey_id);
    println!("ipfsHash: {}", result["Hash"]);
    // done
    store_document_key(dockey_id, &signed_dockey_id, generation_response).unwrap();
}
