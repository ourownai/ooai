/// A reinforcement learning agent using the Q-learning algorithm.
///
/// The `QLearningAgent` struct is designed to learn optimal policies for decision-making within a specified environment.
/// It encapsulates an `Agent` instance that maintains the agent's current state and the action-value function (Q-table).
///
/// # Fields
/// - `agent`: An instance of `Agent` that holds the state and Q-table.
/// - `gamma`: The discount factor for future rewards.
/// - `learning_rate`: The rate at which the agent incorporates new information.
/// - `exploration_rate`: The probability of selecting a random action for exploration.
/// - `batch_size`: The number of experiences to sample from the replay buffer when updating.
/// - `replay_buffer`: A binary heap of `Experience` structs for experience replay.
/// - `eligibility_traces`: A 2D vector for applying updates across state-action pairs.
/// - `softmax_temp`: The temperature parameter for the softmax action selection policy.
///
/// # Methods
/// - `new`: Initializes a new `QLearningAgent` with specified hyperparameters.
/// - `choose_action`: Selects an action from a given state using a softmax probability distribution.
/// - `update_q_values`: Updates the Q-table using a batch of experiences from the replay buffer.
///
/// # Advanced Features
/// - **Experience Replay**: Enhances learning efficiency by revisiting past decisions and outcomes.
/// - **Eligibility Traces**: Aids in faster convergence to optimal policies by tracking visited states and actions.
/// - **Softmax Action Selection**: Provides a nuanced exploration strategy over the simpler epsilon-greedy method.
///
/// # Examples
/// ```
/// let mut q_agent = QLearningAgent::new(num_states, num_actions, gamma, learning_rate, exploration_rate, batch_size, softmax_temp);
/// let action = q_agent.choose_action(current_state, &valid_actions);
/// q_agent.update_q_values();
/// ```
///
/// # Note
/// The exploration rate can be dynamically adjusted to shift from exploration to exploitation as the agent learns.

use crate::agents::base_agent::Agent;
use crate::iam::user::User;
use crate::iam::verifiable_credentials::{VerifiableCredential, CredentialSubject, sign_credential_with_wallet};
use crate::utils::file_storage::{FileStorageError, UploadedFile};

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};

// Define a struct to represent an experience in the replay buffer.
// Includes state, action taken, reward received, next state, and a priority for sampling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    state: usize,
    action: usize,
    reward: f32,
    next_state: usize,
    priority: f32,
}

// Implement ordering for experiences based on their priority.
// This is necessary for storing them in a binary heap.
impl Ord for Experience {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.partial_cmp(&other.priority).unwrap()
    }
}

impl PartialOrd for Experience {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Experience {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Experience {}

// The QLearningAgent struct now includes a replay buffer for experience replay,
// eligibility traces for more nuanced learning, and a softmax temperature for action selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QLearningAgent {
    pub agent: Agent<'static>,
    gamma: f32,
    learning_rate: f32,
    exploration_rate: f32,
    batch_size: usize,
    replay_buffer: BinaryHeap<Experience>,
    eligibility_traces: Vec<Vec<f32>>,
    softmax_temp: f32,
}

impl QLearningAgent {
    // Initialize a new agent with given parameters, including the size of the replay buffer and softmax temperature.
    pub fn new(
        num_states: usize,
        num_actions: usize,
        gamma: f32,
        learning_rate: f32,
        exploration_rate: f32,
        batch_size: usize,
        softmax_temp: f32,
    ) -> Self {
        Self {
            agent: Agent::new(num_states, num_actions),
            gamma,
            learning_rate,
            exploration_rate,
            batch_size,
            replay_buffer: BinaryHeap::new(),
            eligibility_traces: vec![vec![0.0; num_actions]; num_states],
            softmax_temp,
        }
    }

    // Choose an action for a given state using a softmax probability distribution over valid actions.
    // This approach considers the relative value of each action more nuancedly than picking the max value directly.
    pub fn choose_action(&self, state: usize, valid_actions: &[usize]) -> usize {
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < self.exploration_rate {
            // Exploration: choose a random valid action
            let index = rng.gen_range(0..valid_actions.len());
            valid_actions[index]
        } else {
            // Exploitation: choose the best valid action based on softmax distribution
            let q_values = &self.agent.q_table[state];
            let mut softmax_sum = 0.0;
            let mut softmax_probs = vec![0.0; valid_actions.len()];
            for (i, &action) in valid_actions.iter().enumerate() {
                softmax_probs[i] = (q_values[action] / self.softmax_temp).exp();
                softmax_sum += softmax_probs[i];
            }
            let mut rand_val = rng.gen_range(0.0..softmax_sum);
            for (i, &prob) in softmax_probs.iter().enumerate() {
                rand_val -= prob;
                if rand_val <= 0.0 {
                    return valid_actions[i];
                }
            }
            // If no action is selected due to floating-point issues, choose the first valid action
            valid_actions[0]
        }
    }

    // Update Q-values based on a batch of experiences from the replay buffer.
    // This method samples a batch of experiences to update the agent's knowledge.
    pub fn update_q_values(&mut self) {
        if self.replay_buffer.len() < self.batch_size {
            return;
        }
        let mut batch = Vec::new();
        for _ in 0..self.batch_size {
            batch.push(self.replay_buffer.pop().unwrap());
        }
        for experience in &batch {
            let state = experience.state;
            let action = experience.action;
            let reward = experience.reward;
            let next_state = experience.next_state;
            let old_q_value = self.agent.q_table[state][action];
            let next_state_q_values = &self.agent.q_table[next_state];
            let max_next_q_value = next_state_q_values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let td_error = reward + self.gamma * max_next_q_value - old_q_value;
            self.eligibility_traces[state][action] += 1.0;
            // Update Q-values across all states and actions using the eligibility traces,
            // allowing for more effective learning over sequences of actions.
            for s in 0..self.agent.state {
                for a in 0..self.agent.q_table[s].len() {
                    self.agent.q_table[s][a] += self.learning_rate * td_error * self.eligibility_traces[s][a];
                    self.eligibility_traces[s][a] *= self.gamma;
                }
            }
        }
    }

    // Add an experience to the replay buffer with a simple priority scheme based on the absolute reward.
    pub fn add_experience(&mut self, state: usize, action: usize, reward: f32, next_state: usize) {
        let priority = reward.abs(); // Simple priority based on absolute reward
        let experience = Experience {
            state,
            action,
            reward,
            next_state,
            priority,
        };
        self.replay_buffer.push(experience);
    }

    // Dynamically adjust the exploration rate based on the number of iterations,
    // encouraging exploration early on and exploitation later.
    pub fn update_exploration_rate(&mut self, iteration: usize, max_iterations: usize) {
        let min_exploration_rate = 0.01;
        let decay_rate = (self.exploration_rate - min_exploration_rate) / max_iterations as f32;
        self.exploration_rate = (self.exploration_rate - decay_rate * iteration as f32).max(min_exploration_rate);
    }

    // Save the current Q-table to a file.
    pub async fn save_q_table(&self, user: &User) -> Result<String, Box<dyn std::error::Error>> {
        // Encrypt the Q-table using a symmetric encryption algorithm
        let encrypted_q_table = self.encrypt_q_table(user)?;

        // Create a verifiable credential with the encrypted Q-table as the subject
        let vc = VerifiableCredential {
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
                "https://www.w3.org/2018/credentials/examples/v1".to_string(),
            ],
            id: format!("did:example:{}/credentials/q-table", user.id),
            issuer: format!("did:example:{}", user.id),
            issuance_date: chrono::Utc::now().to_string(),
            credential_subject: CredentialSubject {
                id: format!("did:example:{}", user.id),
                wallet_address: user.wallet.get_address(),
            },
            types: vec!["VerifiableCredential".to_string(), "QTableCredential".to_string()],
            proof: None,
        };

        // Sign the verifiable credential using the user's wallet
        let signed_vc = sign_credential_with_wallet(&vc, &user.wallet).await?;

        // Store the signed verifiable credential using the project's internal RPC mechanism
        let uploaded_file = UploadedFile {
            user_id: user.id.clone(),
            data: serde_json::to_string(&signed_vc)?.into_bytes(),
        };

        // Use the project's internal RPC mechanism to store the uploaded file
        let cid = self.store_file(uploaded_file).await?;

        // Associate the CID with the user's DID in a mapping
        let mut q_table_mapping = self.load_q_table_mapping()?;
        q_table_mapping.insert(user.id.clone(), cid.clone());
        self.save_q_table_mapping(&q_table_mapping)?;

        Ok(cid)
    }

    fn encrypt_q_table(&self, user: &User) -> Result<String, FileStorageError> {
        // Implement the logic to encrypt the Q-table using a symmetric encryption algorithm
        // You can derive the encryption key from the user's wallet or store it securely
        let key = user.wallet.get_encryption_key()?;
        let encrypted_data = encrypt_data(&serde_json::to_string(&self.agent.q_table)?, &key)?;
        Ok(base64::engine::general_purpose::STANDARD.encode(&encrypted_data))
    }

    async fn store_file(&self, file: UploadedFile) -> Result<String, Box<dyn std::error::Error>> {
        // Implement the logic to store the file using the project's internal RPC mechanism
        // This could involve calling an internal RPC service or using a shared RPC client
        // For simplicity, let's assume we have a helper function to store the file
        let cid = store_file_via_rpc(file).await?;
        Ok(cid)
    }

    fn load_q_table_mapping(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        // Implement the logic to load the Q-table mapping from a persistent storage
        // This could be a smart contract, a distributed database, or a local file
        // For simplicity, let's assume we have a helper function to load the mapping
        let mapping = load_mapping_from_storage()?;
        Ok(mapping)
    }

    fn save_q_table_mapping(&self, mapping: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        // Implement the logic to save the Q-table mapping to a persistent storage
        // This could be a smart contract, a distributed database, or a local file
        // For simplicity, let's assume we have a helper function to save the mapping
        save_mapping_to_storage(mapping)?;
        Ok(())
    }

    // Load a Q-table from a file, updating the agent's knowledge.
    pub fn load_q_table(&mut self, file_path: &str) -> Result<(), std::io::Error> {
        let file = std::fs::File::open(file_path)?;
        self.agent.q_table = serde_json::from_reader(file)?;
        Ok(())
    }

    // Reset the agent's current state, allowing for a fresh start or setting a specific initial state.
    pub fn reset_agent_state(&mut self, state: Option<usize>) {
        self.agent.state = state.unwrap_or(0);
    }    

    pub fn state(&self) -> usize {
        self.agent.state
    }

    pub fn set_state(&mut self, state: usize) {
        self.agent.state = state;
    }
}

// Helper functions for encryption and storage (to be implemented separately)
fn encrypt_data(data: &str, key: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Implement the logic to encrypt the data using the provided encryption key
    // You can use libraries like `aes-gcm` or `ring` for symmetric encryption
    Ok(vec![])
}

async fn store_file_via_rpc(file: UploadedFile) -> Result<String, Box<dyn std::error::Error>> {
    // Implement the logic to store the file using the project's internal RPC mechanism
    // This could involve calling an internal RPC service or using a shared RPC client
    Ok(String::new())
}

fn load_mapping_from_storage() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // Implement the logic to load the Q-table mapping from a persistent storage
    Ok(HashMap::new())
}

fn save_mapping_to_storage(mapping: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    // Implement the logic to save the Q-table mapping to a persistent storage
    Ok(())
}
