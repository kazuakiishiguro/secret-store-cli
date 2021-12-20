use crate::bytes::Bytes;
use ethereum_types::H512;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct EncryptedDocumentKey {
    pub common_point: H512,
    pub encrypted_point: H512,
    pub encrypted_key: Bytes,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct DecryptedDocumentKey {
    pub decrypted_secret: H512,
    pub common_point: H512,
    pub decrypt_shadows: Vec<Bytes>,
}

#[cfg(test)]
mod tests {
    use super::{EncryptedDocumentKey, H512};
    use serde_json;

    #[test]
    fn test_serialize_encrypted_document_key() {
        let initial = EncryptedDocumentKey {
            common_point: H512::from_low_u64_be(1),
            encrypted_point: H512::from_low_u64_be(2),
            encrypted_key: vec![3].into(),
        };

        let serialized = serde_json::to_string(&initial).unwrap();
        assert_eq!(
            serialized,
            r#"{"common_point":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001","encrypted_point":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002","encrypted_key":"0x03"}"#
        );

        let deserialized: EncryptedDocumentKey = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.common_point, H512::from_low_u64_be(1));
        assert_eq!(deserialized.encrypted_point, H512::from_low_u64_be(2));
        assert_eq!(deserialized.encrypted_key, vec![3].into());
    }
}
