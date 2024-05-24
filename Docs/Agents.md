---
title: Agents Crate
description: A collection of reinforcement learning agents and knowledge graph management tools for Rust.
---

# Agents Crate

The `agents` crate provides a collection of reinforcement learning agents and knowledge graph management tools for building intelligent systems in Rust.

## Features

- **base_agent**: Defines the core `Agent` struct with state and Q-table for reinforcement learning.
- **knowledge_agent**: Extends the `Agent` struct with knowledge graph capabilities for managing and querying domain-specific information.
- **q_learning_agent**: Implements the Q-learning algorithm with experience replay, eligibility traces, and softmax action selection for efficient and effective learning.
- **providers.json**: Contains metadata for external data providers, including cost, copyright, and performance metrics.

## Usage

Add the `agents` crate to your Rust project's dependencies in `Cargo.toml`:

\`\`\`toml
[dependencies]
agents = { path = "path/to/agents" }
\`\`\`

Import the desired agent modules in your Rust code:

\`\`\`rust
use agents::base_agent::Agent;
use agents::knowledge_agent::Agent as KnowledgeAgent;
use agents::q_learning_agent::QLearningAgent;
\`\`\`

Create instances of the agents and use their methods to build, update, and query knowledge graphs or perform reinforcement learning tasks.

\`\`\`rust
let mut agent = KnowledgeAgent::new();
agent.build_knowledge_graph("This is a sample text.");
let matching_nodes = agent.search("sample");

let mut q_agent = QLearningAgent::new(num_states, num_actions, gamma, learning_rate, exploration_rate, batch_size, softmax_temp);
q_agent.update_q_values();
\`\`\`

## Dependencies

The `agents` crate relies on the following dependencies:

- **rand**: For generating random numbers in the Q-learning agent.
- **serde**: For serializing and deserializing agent structs and provider metadata.
- **serde_json**: For parsing the `providers.json` file.

Make sure to include these dependencies in your project's `Cargo.toml` file.

## Provider Metadata

The `providers.json` file contains metadata for external data providers. Each provider entry should follow this format:

\`\`\`json
{
  "name": "Provider Name",
  "provider_type": ["Type1", "Type2"],
  "supported_content_types": ["ContentType1", "ContentType2"],
  "cost_per_request": {
    "amount": 0.01,
    "currency": "USD"
  },
  "copyright_ownership": "Owner",
  "data_reproduction_rights": "Reproduction Rights",
  "data_handling": {
    "storage_duration": "1 year",
    "usage_policy": "Policy"
  },
  "performance_metrics": {
    "accuracy": 0.95,
    "speed": "100ms"
  }
}
\`\`\`

Ensure that the `providers.json` file is properly formatted and located in the `src/agents` directory.

## License

This crate is licensed under the AGPLv3 License.
