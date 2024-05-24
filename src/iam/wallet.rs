use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use web3::types::Address;
use std::sync::Arc;
use ockam_vault::legacy::SecretAttributes;


use crate::clients::kv::{KVStore, MemoryKVStore, PrefixedKVStore};
use crate::iam::did::{DID, resolve, VerifiableCredential};
use crate::iam::public_key_store::PublicKeyStore;
use crate::encryption::encryption::EncryptHandler;
use crate::iam::user_data::UserData;

// Custom struct to represent a wallet address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct WalletAddress(pub Address);

impl From<Address> for WalletAddress {
    fn from(address: Address) -> Self {
        WalletAddress(address)
    }
}

// Implement the Display trait for the DID struct
impl fmt::Display for DID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub id: String,
    pub public_key: String,
    pub did: String,
    pub identity_doc: String,
    pub credentials: HashMap<String, String>,
    pub keys: Vec<String>,
    pub addresses: Vec<WalletAddress>,
    pub preferred_address: WalletAddress,
    pub base_currency: String,
    pub payment_thresholds: HashMap<String, u64>,   
}

impl Wallet {
    // Generate a new DID/DID document
    pub async fn new_wallet() -> Self {
        let (did, id_doc) = DID::generate();
        let store: Arc<dyn KVStore> = Arc::new(MemoryKVStore::default());
        let keyid_store: Arc<dyn KVStore> = Arc::new(PrefixedKVStore::new(store.clone(), "OCKAM_KEYID:".into()));
        let handler = EncryptHandler::new(keyid_store);
        
        // Get or create the KeyId for the DID
        let key_id = handler.get_or_create_keyid(did.to_string(), SecretAttributes::Aes256).await.unwrap();
        
        // Encrypt the identity document using the KeyId
        let enc_doc = handler.aes_encrypt_message(&key_id, id_doc.to_string().as_bytes(), [0u8; 8]).await.unwrap();
        
        Self {
            id: did.to_string(),
            public_key: did.public_key(), // Use the PublicKeyStore trait to retrieve the public key
            did: did.to_string(),
            identity_doc: enc_doc,
            credentials: HashMap::new(),
            keys: vec![],
            addresses: vec![],
            preferred_address: WalletAddress::default(),
            base_currency: "ETH".to_string(),
            payment_thresholds: HashMap::new(),
        }
    }

    pub fn get_address(&self) -> Address {
        self.preferred_address.0
    }

    // Persist verifiable credential
    pub async fn store_vc(&mut self, vc: VerifiableCredential) {
        let store = Arc::new(MemoryKVStore::default());
        let keyid_store = Arc::new(PrefixedKVStore::new(store.clone(), "OCKAM_KEYID:".into()));
        let handler = EncryptHandler::new(keyid_store);
        let encrypted = handler.aes_encrypt_message(&self.did.as_bytes(), serde_json::to_vec(&vc).unwrap().as_slice(), [0u8; 8]).await.unwrap();
        self.credentials.insert(vc.id.clone(), encrypted);
    }

    // Retrieve verifiable credential
    pub async fn get_vc(&self, id: &str) -> Option<VerifiableCredential> {
        self.credentials.get(id).map(|enc| async {
            let store: Arc<dyn KVStore> = Arc::new(MemoryKVStore::default());
            let keyid_store: Arc<dyn KVStore> = Arc::new(PrefixedKVStore::new(store.clone(), "OCKAM_KEYID:".into()));
            let handler = EncryptHandler::new(keyid_store);
            let decrypted = handler.aes_decrypt_message(&self.did.to_string().as_bytes(), enc.as_bytes()).await.unwrap();
            serde_json::from_slice(&decrypted).unwrap()
        })
        .map(|fut| futures::executor::block_on(fut))
    }

    // Sign credential or other verification
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        // Sign using stored keys
        self.keys[0].sign(data)
    }

    // Verify signature
    pub fn verify(&self, signature: &[u8], data: &[u8]) -> bool {
        // Lookup DID verification methods
        let methods = resolve(&self.did).verification_methods;
        // Verify signature using methods
        methods.iter().any(|m| m.verify(signature, data))
    }

    // Add a new wallet address
    pub fn add_address(&mut self, address: Address) {
        self.addresses.push(WalletAddress::from(address));
    }

    pub fn set_preferred_address(&mut self, address: Address) {
        self.preferred_address = WalletAddress::from(address);
    }

    // Set the base currency
    pub fn set_base_currency(&mut self, currency: String) {
        self.base_currency = currency;
    }

    // Set payment thresholds for different currencies
    pub fn set_payment_threshold(&mut self, currency: String, threshold: u64) {
        self.payment_thresholds.insert(currency, threshold);
    }
}

// Function to make a payment with the wallet
pub async fn make_payment_with_wallet(
    wallet: &Wallet,
    to_address: Address,
    amount: u64,
    currency: &str,
    user_data: &UserData,
) -> Result<String, String> {
    // Check if the wallet has sufficient funds
    let balance = get_wallet_balance(wallet, currency, user_data).await.map_err(|e| e.to_string())?;
    if balance < amount {
        return Err(format!("Insufficient funds in the wallet for currency: {}", currency));
    }

    // Check if the payment amount exceeds the threshold for the currency
    if let Some(threshold) = wallet.payment_thresholds.get(currency) {
        if amount > *threshold {
            return Err(format!("Payment amount exceeds the threshold for currency: {}", currency));
        }
    }

    // Determine the wallet address to use for the payment
    let from_address = if wallet.addresses.contains(&WalletAddress::from(wallet.preferred_address.0)) {
        wallet.preferred_address.0
    } else {
        // Use a distributed approach to select the from address
        let index = calculate_distributed_index(wallet, amount, user_data);
        wallet.addresses[index].0
    };

    // Sign the payment transaction
    let tx_data = create_transaction_data(from_address, to_address, amount, currency, user_data);
    let signature = wallet.sign(&tx_data);

    // Send the payment transaction
    let tx_hash = send_transaction(from_address, to_address, amount, currency, signature, user_data).await.map_err(|e| e.to_string())?;
    Ok(tx_hash)
}

// Custom struct to represent user preferences, profiles, and histories
struct UserWalletData {
    preferences: HashMap<String, String>,
    profile: HashMap<String, String>,
    history: Vec<(SystemTime, String)>,
}

// Function to calculate the distributed index for selecting the from address
fn calculate_distributed_index(wallet: &Wallet, amount: u64, user_data: &UserData) -> usize {
    // Create a deterministic seed based on the payment amount, user preferences, and current time
    let seed = format!(
        "{}-{}-{:?}",
        amount,
        user_data.preferences.get("payment_seed").unwrap_or(&"default_seed".to_string()),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
    );

    // Create a HashMap to store the distribution weights for each address
    let mut weights = HashMap::new();

    // Calculate the distribution weight for each address based on user profile and history
    for (i, address) in wallet.addresses.iter().enumerate() {
        let profile_weight = user_data.profile.get(&format!("address_{}", i)).unwrap_or(&"1".to_string()).parse::<u64>().unwrap_or(1);
        let history_weight = user_data.history.iter().filter(|(_, addr)| addr == &address.0).count() as u64;
        let total_weight = profile_weight * history_weight;
        weights.insert(address, total_weight);
    }

    // Calculate the total weight sum
    let total_weight_sum: u64 = weights.values().sum();

    // Generate a random value based on the seed
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    seed.hash(&mut hasher);
    let random_value = hasher.finish() % total_weight_sum;

    // Select the address based on the random value and distribution weights
    let mut cumulative_weight = 0;
    for (i, (address, weight)) in weights.iter().enumerate() {
        cumulative_weight += weight;
        if random_value < cumulative_weight {
            return i;
        }
    }

    // Fallback to the last address if no address is selected
    wallet.addresses.len() - 1
}

// Function to get the balance of a specific currency in the wallet
async fn get_wallet_balance(wallet: &Wallet, currency: &str, user_data: &UserData) -> Result<u64, String> {
    // Check if the user has a preferred balance provider for the currency
    if let Some(provider) = user_data.preferences.get(&format!("balance_provider_{}", currency)) {
        // Use the specified balance provider API to retrieve the balance
        let balance = match provider.as_str() {
            "solana" => retrieve_balance_from_solana(wallet, currency).await?,
            "ethereum" => retrieve_balance_from_ethereum(wallet, currency).await?,
            "polkadot" => retrieve_balance_from_polkadot(wallet, currency).await?,
            _ => return Err(format!("Unknown balance provider: {}", provider)),
        };
        return Ok(balance);
    }

    // Fallback to retrieving the balance from the default blockchain API
    retrieve_balance_from_default(wallet, currency).await
}

// Function to create the transaction data for a payment
fn create_transaction_data(from: Address, to: Address, amount: u64, currency: &str, user_data: &UserData) -> Vec<u8> {
    // Create a HashMap to store the transaction data
    let mut tx_data = HashMap::new();
    tx_data.insert("from".to_string(), format!("{:?}", from));
    tx_data.insert("to".to_string(), format!("{:?}", to));
    tx_data.insert("amount".to_string(), amount.to_string());
    tx_data.insert("currency".to_string(), currency.to_string());

    // Add user-specific metadata to the transaction data
    if let Some(metadata) = user_data.preferences.get("tx_metadata") {
        tx_data.insert("metadata".to_string(), metadata.clone());
    }

    // Serialize the transaction data into a byte vector
    serde_json::to_vec(&tx_data).unwrap_or(vec![])
}

// Function to send the payment transaction
async fn send_transaction(from: Address, to: Address, amount: u64, currency: &str, signature: Vec<u8>, user_data: &UserData) -> Result<String, String> {
    // Check if the user has a preferred transaction provider
    if let Some(provider) = user_data.preferences.get("tx_provider") {
        // Use the specified transaction provider API to send the transaction
        let tx_hash = match provider.as_str() {
            "solana" => send_transaction_with_solana(from, to, amount, currency, signature).await?,
            "ethereum" => send_transaction_with_ethereum(from, to, amount, currency, signature).await?,
            "polkadot" => send_transaction_with_polkadot(from, to, amount, currency, signature).await?,
            _ => return Err(format!("Unknown transaction provider: {}", provider)),
        };
        return Ok(tx_hash);
    }

    // Fallback to sending the transaction using the default blockchain API
    send_transaction_with_default(from, to, amount, currency, signature).await
}

// Solana balance retrieval and transaction sending
async fn retrieve_balance_from_solana(wallet: &Wallet, currency: &str) -> Result<u64, String> {
    // Use the Solana JSON-RPC API to retrieve the balance
    // Solana features:
    // - Fast and scalable blockchain with high throughput
    // - Parallel transaction processing using Proof of History (PoH)
    // - Support for multiple tokens and SPL token standard
    // Example JSON-RPC request:
    // {"jsonrpc":"2.0","id":1,"method":"getBalance","params":[wallet.addresses[0].to_string()]}
    // Parse the JSON-RPC response and extract the balance
    Ok(1000) // Dummy balance
}

async fn send_transaction_with_solana(from: Address, to: Address, amount: u64, currency: &str, signature: Vec<u8>) -> Result<String, String> {
    // Use the Solana JSON-RPC API to send the transaction
    // Solana features:
    // - Fast transaction confirmation times
    // - Low transaction fees
    // - Support for multiple transaction types (e.g., transfer, vote, stake)
    // Example JSON-RPC request:
    // {"jsonrpc":"2.0","id":1,"method":"sendTransaction","params":[transaction_data]}
    // Parse the JSON-RPC response and extract the transaction hash
    Ok("solana_tx_hash".to_string()) // Dummy transaction hash
}

// Ethereum balance retrieval and transaction sending
async fn retrieve_balance_from_ethereum(wallet: &Wallet, currency: &str) -> Result<u64, String> {
    // Use the Ethereum JSON-RPC API or Web3 library to retrieve the balance
    // Ethereum features:
    // - Smart contract functionality using Solidity
    // - Decentralized application (dApp) development
    // - Support for ERC20 and other token standards
    // Example JSON-RPC request:
    // {"jsonrpc":"2.0","id":1,"method":"eth_getBalance","params":[wallet.addresses[0].to_string(), "latest"]}
    // Parse the JSON-RPC response and extract the balance
    Ok(1500) // Dummy balance
}

async fn send_transaction_with_ethereum(from: Address, to: Address, amount: u64, currency: &str, signature: Vec<u8>) -> Result<String, String> {
    // Use the Ethereum JSON-RPC API or Web3 library to send the transaction
    // Ethereum features:
    // - Gas-based transaction fees
    // - Support for smart contract interactions
    // - Decentralized finance (DeFi) ecosystem
    // Example JSON-RPC request:
    // {"jsonrpc":"2.0","id":1,"method":"eth_sendRawTransaction","params":[signed_transaction]}
    // Parse the JSON-RPC response and extract the transaction hash
    Ok("ethereum_tx_hash".to_string()) // Dummy transaction hash
}

// Polkadot balance retrieval and transaction sending
async fn retrieve_balance_from_polkadot(wallet: &Wallet, currency: &str) -> Result<u64, String> {
    // Use the Polkadot JSON-RPC API or Substrate API client to retrieve the balance
    // Polkadot features:
    // - Multichain architecture with parachains
    // - Shared security and interoperability between chains
    // - Governance and on-chain upgrades
    // Example JSON-RPC request:
    // {"jsonrpc":"2.0","id":1,"method":"account_getBalance","params":[wallet.addresses[0].to_string()]}
    // Parse the JSON-RPC response and extract the balance
    Ok(2000) // Dummy balance
}

async fn send_transaction_with_polkadot(from: Address, to: Address, amount: u64, currency: &str, signature: Vec<u8>) -> Result<String, String> {
    // Use the Polkadot JSON-RPC API or Substrate API client to send the transaction
    // Polkadot features:
    // - Cross-chain token transfers
    // - Support for custom runtime modules and pallets
    // - Staking and governance mechanisms
    // Example JSON-RPC request:
    // {"jsonrpc":"2.0","id":1,"method":"author_submitExtrinsic","params":[signed_transaction]}
    // Parse the JSON-RPC response and extract the transaction hash
    Ok("polkadot_tx_hash".to_string()) // Dummy transaction hash
}

// Default balance retrieval and transaction sending
async fn retrieve_balance_from_default(wallet: &Wallet, currency: &str) -> Result<u64, String> {
    // Retrieve the balance from the default blockchain API
    Ok(2500) // Dummy balance
}

async fn send_transaction_with_default(from: Address, to: Address, amount: u64, currency: &str, signature: Vec<u8>) -> Result<String, String> {
    // Send the transaction using the default blockchain API
    Ok("default_tx_hash".to_string()) // Dummy transaction hash
}