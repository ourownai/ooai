use crate::iam::did::DID;

pub trait PublicKeyStore {
    fn public_key(&self) -> String;
}

impl PublicKeyStore for DID {
    fn public_key(&self) -> String {
        // Implement the logic to retrieve the public key from the DID
        // and return it as a String
        // Example:
        self.public_key.clone()
    }
}
