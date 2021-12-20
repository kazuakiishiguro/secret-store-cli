use std::io::Result;

use ethereum_types::H256;
use serde_json::{json, Value};
use url::Url;

use crate::dependency::Dependencies;

fn retrieve_document_keys(dockey_id: H256, signed_dockey_id: &str) -> Result<String> {
    let base = Url::parse("http://localhost:8000/shadow/").unwrap();
    let path = format!(
        "{}{}{}{}",
        &hex::encode(H256::as_bytes(&dockey_id)),
        "/",
        signed_dockey_id,
        "?apikey=fa05a2e1-d323-4723-96b7-4d2695a61d3f"
    );
    let url = base.join(&path).unwrap();
    let mut buf: Vec<u8> = vec![];
    let mut res = reqwest::get(url.as_str()).unwrap();
    res.copy_to(&mut buf).unwrap();
    Ok(String::from_utf8(buf.to_vec()).unwrap())
}

pub fn decrypt(dockey_id: H256, ipfs_uri: String) {
    let deps = Dependencies::new();
    let io = deps.default_client();

    // TODO: should be replaced with 'select account and download encrypted file
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
    let decryption_keys: Value =
        serde_json::from_str(&retrieve_document_keys(dockey_id, &signed_dockey_id).unwrap())
            .unwrap();
    let encryption_response = deps.cat_ipfs(&ipfs_uri).unwrap();
    let encryption_response = encryption_response.replace("\"", "");
    let params = json!({
        "jsonrpc": "2.0",
        "method": "secretstore_shadowDecrypt",
        "params": [
            &deps.address,
            &deps.password,
            &decryption_keys["decrypted_secret"],
            &decryption_keys["common_point"],
            &decryption_keys["decrypt_shadows"],
            &encryption_response,
        ],
        "id": 1,
    });
    let decrypted_document = io.handle_request_sync(&params.to_string()).unwrap();
    let decrypted_document = decrypted_document.replace(r#"{"jsonrpc":"2.0","result":"0x"#, "");
    let decrypted_document = decrypted_document.replace(r#"","id":1}"#, "");
    let decoded_string = String::from_utf8(hex::decode(decrypted_document).unwrap()).unwrap();
    let document = deps.decode_reader(&decoded_string).unwrap();
    println!(
        "{}",
        String::from_utf8(document).expect("file could not convert to document")
    );
}
