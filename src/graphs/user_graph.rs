use crate::iam::user::User;
use crate::iam::group::Group;
use crate::agents::q_learning_agent::QLearningAgent;
use crate::messaging::message::Message;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::collections::HashSet;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

const INITIAL_EXPLORATION_RATE: f32 = 1.0;
const MAX_ITERATIONS: usize = 1000;
const EARLY_STOPPING_THRESHOLD: f32 = 0.01;
const MIN_EXPLORATION_RATE: f32 = 0.1;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGraph {
    pub nodes: Vec<Node>,
    pub users: Vec<User>,
    pub groups: Vec<Group>,
}

// Enum to represent different types of nodes in the user graph
enum UserNodeType {
    Intent(String),
    Entity(String),
    Preference(String),
}

// Struct to represent a node in the user graph
pub struct UserNode {
    pub node_type: UserNodeType,
    pub values: HashSet<String>,
    pub embeddings: Vec<f32>,
    pub timestamp: SystemTime,
}

impl UserNode {
    // Create a new user node
    fn new(node_type: UserNodeType, values: HashSet<String>, embeddings: Vec<f32>, timestamp: SystemTime) -> Self {
        UserNode {
            node_type,
            values,
            embeddings,
            timestamp,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub messages: Vec<Message>,
    pub user_id: Option<usize>,
    pub group_id: Option<usize>,
    pub reward: f32,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Edge {
    pub weight: f32,
    pub to: usize,
    pub reward: f32,
}

impl UserGraph {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let user_graph: UserGraph = serde_json::from_reader(reader)?;
        Ok(user_graph)
    }

    pub fn train_q_learning_agent(&mut self, num_iterations: usize) {
        let num_states = self.users.len();
        let num_actions = self.users.len();
        let gamma = 0.9;
        let learning_rate = 0.1;
        let mut q_agent = QLearningAgent::new(
            num_states,
            num_actions,
            gamma,
            learning_rate,
            INITIAL_EXPLORATION_RATE,
            32,
            1.0,
        );

        let mut best_reward = 0.0;
        let mut best_q_table = q_agent.agent.q_table.clone();
        let mut num_iterations_without_improvement = 0;

        for i in 0..num_iterations {
            let mut total_reward = 0.0;
            q_agent.reset_state(None); // Reset state to the starting state
    
            loop {
                let current_state = q_agent.state();
                let valid_actions: Vec<usize> = self.users.iter().enumerate()
                    .filter(|(_, user)| self.can_transition(current_state, user.id))
                    .map(|(index, _)| index)
                    .collect();

                if valid_actions.is_empty() {
                    break;
                }

                let action = q_agent.choose_action(current_state, &valid_actions);
                let next_state = action;
                let reward = self.calculate_reward(current_state, next_state);
                total_reward += reward;

                q_agent.add_experience(current_state, action, reward, next_state);
                q_agent.update_q_values();
                q_agent.set_state(next_state);
            }

            if total_reward > best_reward {
                best_reward = total_reward;
                best_q_table = q_agent.agent.q_table.clone();
                num_iterations_without_improvement = 0;
            } else {
                num_iterations_without_improvement += 1;
            }

            if num_iterations_without_improvement >= EARLY_STOPPING_THRESHOLD as usize {
                break;
            }

            q_agent.update_exploration_rate(i, MAX_ITERATIONS);
        }

        q_agent.agent.q_table = best_q_table;
        let file_path = "best_q_table.json";
        q_agent.save_q_table(file_path).expect("Failed to save Q-table");
    }

    pub fn get_top_messages_by_feedback(&self, limit: usize) -> Vec<&Message> {
        let mut messages: Vec<&Message> = self.nodes.iter().flat_map(|node| &node.messages).collect();
        messages.sort_by(|a, b| {
            let total_feedback_a: f32 = a.feedback_weights.iter().sum();
            let total_feedback_b: f32 = b.feedback_weights.iter().sum();
            total_feedback_b.partial_cmp(&total_feedback_a).unwrap()
        });
        messages.into_iter().take(limit).collect()
    }

    pub fn get_top_nodes_by_feedback(&self, limit: usize) -> Vec<&Node> {
        let mut nodes: Vec<&Node> = self.nodes.iter().collect();
        nodes.sort_by(|a, b| {
            let total_feedback_a: f32 = a.messages.iter().flat_map(|message| &message.feedback_weights).sum();
            let total_feedback_b: f32 = b.messages.iter().flat_map(|message| &message.feedback_weights).sum();
            total_feedback_b.partial_cmp(&total_feedback_a).unwrap()
        });
        nodes.into_iter().take(limit).collect()
    }

    pub fn get_connected_nodes(&self, node_index: usize) -> Vec<&Node> {
        let node = &self.nodes[node_index];
        let connected_indices: Vec<usize> = node.messages.iter().flat_map(|message| &message.to_edges).map(|edge| edge.to).collect();
        connected_indices.into_iter().map(|index| &self.nodes[index]).collect()
    }

    pub fn get_strongly_connected_components(&self) -> Vec<Vec<usize>> {
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = Vec::new();
        let mut components = Vec::new();

        for i in 0..self.nodes.len() {
            if !visited[i] {
                self.dfs(i, &mut visited, &mut stack);
            }
        }

        visited.fill(false);
        while let Some(node_index) = stack.pop() {
            if !visited[node_index] {
                let mut component = Vec::new();
                self.dfs_reverse(node_index, &mut visited, &mut component);
                components.push(component);
            }
        }

        components
    }

    fn dfs(&self, node_index: usize, visited: &mut Vec<bool>, stack: &mut Vec<usize>) {
        visited[node_index] = true;
        let node = &self.nodes[node_index];
        for edge in &node.edges {
            if !visited[edge.to] {
                self.dfs(edge.to, visited, stack);
            }
        }
        stack.push(node_index);
    }

    fn dfs_reverse(&self, node_index: usize, visited: &mut Vec<bool>, component: &mut Vec<usize>) {
        visited[node_index] = true;
        component.push(node_index);
        let node = &self.nodes[node_index];
        for edge in &node.edges {
            if !visited[edge.to] {
                self.dfs_reverse(edge.to, visited, component);
            }
        }
    }
}

pub fn run_reinforcement_learning(user_graph: &mut UserGraph) -> Result<(), std::io::Error> {
    let num_states = user_graph.nodes.len();
    let num_actions = user_graph.nodes.get(0).map_or(0, |n| n.messages.len());
    let gamma = 0.9;
    let learning_rate = 0.1;
    let mut agent = QLearningAgent::new(num_states, num_actions, gamma, learning_rate, INITIAL_EXPLORATION_RATE, 32, 1.0);
    let mut num_iterations = 0;
    let mut last_total_reward = 0.0;
    let mut improvement = f32::INFINITY;
    let mut exploration_rate = INITIAL_EXPLORATION_RATE;

    while agent.agent.state != num_states - 1 && num_iterations < MAX_ITERATIONS && improvement > EARLY_STOPPING_THRESHOLD {
        num_iterations += 1;
        exploration_rate = update_exploration_rate(num_iterations);
        
        // Define valid_actions
        let valid_actions: Vec<usize> = user_graph.nodes[agent.agent.state].edges.iter().map(|edge| edge.to).collect();
        
        let action = agent.choose_action(agent.agent.state, &valid_actions);
        let (next_state, reward) = simulate_action(user_graph, &agent, action);
        let feedback_text = read_message(user_graph, &agent, action);
        let feedback = process_feedback(&feedback_text);
        update_message_feedback(user_graph, &agent, action, &feedback, num_iterations);
        agent.update_q_value(reward, next_state, action);
        agent.agent.state = next_state;

        if agent.agent.state == num_states - 1 || agent.agent.state == 0 {
            agent.reset_state(None); // Reset state to the starting state
            let current_total_reward = calculate_total_reward(user_graph);
            improvement = (last_total_reward - current_total_reward).abs();
            last_total_reward = current_total_reward;
            log_progress(num_iterations, current_total_reward, exploration_rate);
        }
    }

    agent.save_q_table("q_table.json")?;
    Ok(())
}

fn update_exploration_rate(iteration: usize) -> f32 {
    let decay_rate = (INITIAL_EXPLORATION_RATE - MIN_EXPLORATION_RATE) / MAX_ITERATIONS as f32;
    (INITIAL_EXPLORATION_RATE - decay_rate * iteration as f32).max(MIN_EXPLORATION_RATE)
}

fn update_message_feedback(user_graph: &mut UserGraph, agent: &QLearningAgent, action: usize, feedback: &[f32], num_iterations: usize) {
    if let Some(node) = user_graph.nodes.get_mut(agent.agent.state) {
        if let Some(message) = node.messages.get_mut(action) {
            for (i, &value) in feedback.iter().enumerate() {
                message.feedback_weights[i] += value / num_iterations as f32;
            }
        }
    }
}

fn log_progress(iteration: usize, total_reward: f32, exploration_rate: f32) {
    println!("Iteration: {}, Total Reward: {:.2}, Exploration Rate: {:.2}", iteration, total_reward, exploration_rate);
}

pub fn calculate_total_reward(user_graph: &UserGraph) -> f32 {
    user_graph.nodes.iter().map(|node| node.reward).sum()
}

fn simulate_action(user_graph: &UserGraph, agent: &QLearningAgent, action: usize) -> (usize, f32) {
    if let Some(node) = user_graph.nodes.get(agent.agent.state) {
        if let Some(edge) = node.edges.get(action) {
            return (edge.to, edge.reward);
        }
    }
    (agent.agent.state, 0.0)
}

fn read_message(user_graph: &UserGraph, agent: &QLearningAgent, action: usize) -> String {
    if let Some(node) = user_graph.nodes.get(agent.agent.state) {
        if let Some(message) = node.messages.get(action) {
            return message.text.clone();
        }
    }
    String::new()
}

fn process_feedback(feedback_text: &str) -> Vec<f32> {
    // Implement logic to process feedback text and convert it into a vector of feedback weights
    vec![0.0] // Placeholder
}

impl QLearningAgent {
    fn reset_state(&mut self, initial_state: Option<usize>) {
        self.agent.state = initial_state.unwrap_or(0); // Reset state to the starting state or the provided initial state
    }
}