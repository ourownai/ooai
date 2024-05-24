/*
This module introduces a function named "run_reinforcement_learning" designed to execute a reinforcement learning process on a "UserGraph" structure. This function accepts a mutable reference to a "UserGraph" object and employs a Q-learning algorithm to optimize the decision-making policy within the graph. The return type of the function is a Result object to handle potential errors during execution.

Key functionalities of this process include:

1. Initialization of a Q-learning agent, configured with the user graph's dimensions (number of states and actions) and learning parameters (gamma, learning rate, exploration rate).

2. Execution of a learning loop where the agent:
   - Observes the current state and selects an action based on an epsilon-greedy strategy, facilitated by the "choose_action" method.
   - Simulates the chosen action within the user graph context using the "simulate_action" method, and acquires feedback through the "read_message" method.
   - Processes the feedback to update the feedback weights for the current action's message.
   - Updates the Q-value for the observed state-action pair with the "update_q_value" method, incorporating the received reward and anticipated future rewards.
   - Progresses to the next state, with the capability to reset the agent's state upon reaching a terminal condition.

3. Adaptation of the exploration rate over time to balance the trade-off between exploration of the state-action space and exploitation of known values.

4. Implementation of an early stopping mechanism based on the stabilization of the learning process, indicated by minimal improvement in total rewards.

5. Logging of learning progress metrics such as iteration count, total rewards, and current exploration rate for monitoring and analysis purposes.

The learning loop proceeds until a terminal condition is met, which could be due to convergence (minimal improvement), reaching a maximum number of iterations, or encountering an operational error.

Constants for the learning parameters (gamma, learning rate, and initial exploration rate) are defined within the module, supporting the configuration and tuning of the learning process.
*/

use std::collections::HashMap;
use std::io;
use serde::{Deserialize, Serialize};

use crate::agents::q_learning_agent::QLearningAgent;
use crate::graphs::user_graph::UserGraph;
use crate::iam::user::User;
use crate::messaging::message::Message;


const INITIAL_EXPLORATION_RATE: f32 = 0.1;
const MIN_EXPLORATION_RATE: f32 = 0.01;
const MAX_ITERATIONS: usize = 10000;
const EARLY_STOPPING_THRESHOLD: f32 = 0.01;

#[derive(Debug, Serialize, Deserialize)]
pub struct RLHFConfig {
    pub gamma: f32,
    pub learning_rate: f32,
    pub initial_exploration_rate: f32,
    pub min_exploration_rate: f32,
    pub max_iterations: usize,
    pub early_stopping_threshold: f32,
}

impl Default for RLHFConfig {
    fn default() -> Self {
        RLHFConfig {
            gamma: 0.9,
            learning_rate: 0.1,
            initial_exploration_rate: INITIAL_EXPLORATION_RATE,
            min_exploration_rate: MIN_EXPLORATION_RATE,
            max_iterations: MAX_ITERATIONS,
            early_stopping_threshold: EARLY_STOPPING_THRESHOLD,
        }
    }
}

pub fn run_reinforcement_learning(user_graph: &mut UserGraph, config: &RLHFConfig, user: &User) -> Result<(), io::Error> {
    let num_states = user_graph.nodes.len();
    let num_actions = user_graph.nodes.get(0).map_or(0, |n| n.messages.len());
    let mut agent = QLearningAgent::new(
        num_states,
        num_actions,
        config.gamma,
        config.learning_rate,
        config.initial_exploration_rate,
        32,  // Batch size for experience replay
        1.0,  // Softmax temperature for action selection
    );

    let mut num_iterations = 0;
    let mut last_total_reward = 0.0;
    let mut improvement = f32::INFINITY;
    let mut exploration_rate = config.initial_exploration_rate;

    while agent.state() != num_states - 1 && num_iterations < config.max_iterations && improvement > config.early_stopping_threshold {
        num_iterations += 1;
        exploration_rate = update_exploration_rate(num_iterations, config);
        let valid_actions = get_valid_actions(user_graph, &agent);
        let action = agent.choose_action(agent.state(), &valid_actions);
        let (next_state, reward) = simulate_action(user_graph, &agent, action);
        let feedback_text = read_message(user_graph, &agent, action);
        let feedback = process_feedback(&feedback_text);
        update_message_feedback(user_graph, &agent, action, &feedback, num_iterations);
        agent.add_experience(agent.state(), action, reward, next_state);
        agent.update_q_values();
        agent.set_state(next_state);

        if agent.state() == num_states - 1 || agent.state() == 0 {
            agent.reset_agent_state(None);
            let current_total_reward = calculate_total_reward(user_graph);
            improvement = (last_total_reward - current_total_reward).abs();
            last_total_reward = current_total_reward;
            log_progress(num_iterations, current_total_reward, exploration_rate);
        }
    }

    agent.save_q_table(user)?;
    Ok(())
}

fn update_exploration_rate(iteration: usize, config: &RLHFConfig) -> f32 {
    let decay_rate = (config.initial_exploration_rate - config.min_exploration_rate) / config.max_iterations as f32;
    (config.initial_exploration_rate - decay_rate * iteration as f32).max(config.min_exploration_rate)
}

fn log_progress(iteration: usize, total_reward: f32, exploration_rate: f32) {
    println!("Iteration: {}, Total Reward: {:.2}, Exploration Rate: {:.2}", iteration, total_reward, exploration_rate);
}

fn calculate_total_reward(user_graph: &UserGraph) -> f32 {
    user_graph.nodes.iter().map(|node| node.reward).sum()
}

fn simulate_action(user_graph: &UserGraph, agent: &QLearningAgent, action: usize) -> (usize, f32) {
    if let Some(node) = user_graph.nodes.get(agent.state()) {
        if let Some(edge) = node.edges.get(action) {
            return (edge.to, edge.reward);
        }
    }
    (agent.state(), 0.0)
}

fn read_message(user_graph: &UserGraph, agent: &QLearningAgent, action: usize) -> String {
    if let Some(node) = user_graph.nodes.get(agent.state()) {
        if let Some(channel) = &node.channel {
            if let Some(message) = channel.messages.get(action) {
                return message.text.clone();
            }
        }
    }
    String::new()
}


fn process_feedback(feedback_text: &str) -> Vec<f32> {
    // Implement logic to process feedback text and convert it into a vector of feedback weights
    // This could involve sentiment analysis, keyword extraction, or other NLP techniques
    // For simplicity, let's assume the feedback is a comma-separated list of floating-point values
    feedback_text.split(',').map(|s| s.trim().parse().unwrap_or(0.0)).collect()
}

fn get_valid_actions(user_graph: &UserGraph, agent: &QLearningAgent) -> Vec<usize> {
    if let Some(node) = user_graph.nodes.get(agent.state()) {
        (0..node.messages.len()).collect()
    } else {
        vec![]
    }
}

fn update_message_feedback(user_graph: &mut UserGraph, agent: &QLearningAgent, action: usize, feedback: &[f32], num_iterations: usize) {
    if let Some(node) = user_graph.nodes.get_mut(agent.state()) {
        if let Some(channel) = &mut node.channel {
            if let Some(message) = channel.messages.get_mut(action) {
                for (i, &value) in feedback.iter().enumerate() {
                    message.feedback_weights[i] += value / num_iterations as f32;
                }
            }
        }
    }
}
