# Flows Crate Documentation

## Overview

The flows crate provides a framework for defining and executing workflows or flows in a Rust application. It allows you to create complex flows using a combination of predefined block types and custom blocks, enabling the construction of dynamic algorithms and workflows.

The crate consists of several key components:

- `blocks.rs`: Defines the `Block` enum and its variants, along with their associated properties and methods.
- `flowgorithm.rs`: Defines the `Flowgorithm` struct and its methods for processing user instructions, generating logic, and creating flows.
- `blocks.rs`: Defines the `Block` trait and its implementations for different block types.
- `block_library.rs`: Defines the `BlockLibrary` struct for storing and retrieving blocks.
- `logic/scheduling_logic.json`: Provides an example of scheduling logic definition in JSON format.
- `sample_flow.json`: Provides an example of a flow definition in JSON format.
- `flows.rs`: Defines the `FlowEngine` struct and its methods for executing flows.
- `blocks.json`: Defines the JSON schema for the different block types.
- `flow_template.json`: Provides a template for defining flows in JSON format.

## Flows Crate Usage

To use the flows crate in your Rust application, follow these steps:

1. Define your custom blocks in `blocks.rs` by implementing the `Block` trait for each block type. You can also use the predefined block types such as `InputBlock`, `DecisionBlock`, `GoToBlock`, etc.

2. Create a `block_library.json` file that defines a library of reusable blocks. Each block in the library should have a unique identifier and specify its properties, methods, and API integration details.

3. Define the JSON schema for your custom block types in `blocks.json`. This schema specifies the structure and properties of each block type.

4. Create a `flow_template.json` file that provides a template for defining flows. The template should include placeholders for the flow name, start block ID, and blocks.

5. Define your flows in JSON format using the structure specified in `flow_template.json`. Each flow should have a unique name, a start block ID, and an array of blocks. Refer to `sample_flow.json` for an example flow definition.

6. Implement the `Flowgorithm` struct and its methods in `flowgorithm.rs`. The `Flowgorithm` struct is responsible for processing user instructions, generating logic, and creating flows based on the provided block library and flow definitions.

7. Use the `FlowEngine` struct defined in `flows.rs` to execute your flows. The `FlowEngine` takes the flow definitions and a graph of block connections as input and provides methods for executing flows and processing blocks.

8. If your flows involve scheduling logic, you can define the processing logic for scheduling-related blocks in `logic/scheduling_logic.json`. This file specifies the inputs, outputs, and dependencies of each block involved in the scheduling process.

## Example Usage

Here's an example of how to use the flows crate in your Rust application:

```rust
use flows::blocks::{Block, InputBlock, DecisionBlock, GoToBlock};
use flows::flowgorithm::Flowgorithm;
use flows::flows::{FlowEngine, load_flow_definitions, load_graph};

fn main() {
    // Load flow definitions and graph from JSON files
    let flow_definitions = load_flow_definitions("sample_flow.json").unwrap();
    let graph = load_graph("graph.json").unwrap();

    // Create an instance of Flowgorithm
    let mut flowgorithm = Flowgorithm::new();

    // Process user instructions and generate flows
    let user_instruction = "Schedule an appointment with John for next Monday at 10am";
    let result = tokio::runtime::Runtime::new().unwrap().block_on(flowgorithm.process_user_instruction(user_instruction));

    // Create an instance of FlowEngine
    let mut engine = FlowEngine::new(flow_definitions, graph);

    // Calculate block weights for each flow definition
    for flow_definition in engine.flow_definitions.values_mut() {
        engine.calculate_block_weights(flow_definition);
    }

    // Execute the generated flow
    let input_data = HashMap::new();
    let result = tokio::runtime::Runtime::new().unwrap().block_on(engine.execute_flow("example_flow", input_data));

    match result {
        Ok(()) => println!("Flow executed successfully"),
        Err(e) => println!("Flow execution failed: {}", e),
    }
}
```

In this example:

1. We load the flow definitions and graph from JSON files using the `load_flow_definitions` and `load_graph` functions.
2. We create an instance of `Flowgorithm` and process user instructions to generate flows.
3. We create an instance of `FlowEngine` with the loaded flow definitions and graph.
4. We calculate the block weights for each flow definition using the `calculate_block_weights` method.
5. We execute the generated flow using the `execute_flow` method, providing the flow name and input data.
6. Finally, we handle the result of the flow execution, printing a success or failure message.

## Flow Definition Overview

A flow is defined in a JSON format and consists of various blocks, each with a specific role. The flow supports weighted decision-making to allow for probabilistic path selection.

### Example Flow JSON Structure

An example flow named "Schedule Appointment" is provided. It includes six blocks: an input block, an OAuth provider block, a schedule appointment block, a check availability block, a compute slots block, and an end block.

### Flow Definition Template JSON

For creating new flows, use the following template JSON structure, replacing `flow_name`, `block_id`, `BlockType`, and other placeholders with actual values:

```json
{
  "flow_name": {
    "name": "Flow Name",
    "start_block_id": "block_id",
    "blocks": [
      {
        "id": "block_id",
        "type": "BlockType",
        "properties": {
          "property_key": "property_value"
        },
        "connections": {
          "next_block_id": "block_id"
        },
        "weights": {
          "block_id": 0.5
        }
      }
    ]
  }
}
```

## `blocks.rs`

The `blocks.rs` file defines the `Block` enum and its variants, representing different types of blocks that can be used in a flow. The main block types are:

- `InputBlock`: Represents a block that accepts user input.
- `DecisionBlock`: Represents a block that makes a decision based on certain conditions.
- `GoToBlock`: Represents a block that jumps to a specific block in the flow.
- `ConditionalBlock`: Represents a block that executes a specific action based on a condition.
- `DisplayBlock`: Represents a block that displays information to the user.
- `RandomBlock`: Represents a block that selects a random path in the flow.
- `InteractiveBlock`: Represents a block that allows user interaction.
- `ExternalDataBlock`: Represents a block that retrieves data from an external source.

Each block variant has its own set of properties and methods. The `Block` enum is tagged with `#[serde(tag = "type")]` to enable deserialization based on the "type" field in the JSON representation.

## `flows.rs`

The `flows.rs` file defines the `FlowEngine` struct and its methods for executing flows. The `FlowEngine` struct contains the flow definition and provides methods for loading flows, processing blocks, and executing the flow.

The `load_flow` method loads a flow definition from a JSON file and initializes the `FlowEngine` with the loaded flow.

The `process_block` method processes a specific block based on its type, using the corresponding `process_*` methods for each block type.

The `execute` method executes the flow by starting from the initial block and iteratively processing blocks until a terminal condition is reached.

## `blocks.json`

The `blocks.json` file defines the JSON schema for the different block types. It specifies the structure and properties of each block type, including their required and optional fields.

The schema defines the overall structure of a block, with common properties such as `id`, `type`, `properties`, `connections`, and `weights`. It also defines specific properties for each block type, such as `api_integration` and `parameters_schema` for `InputBlock`, and `graph_weights` for `DecisionBlock`.

## `sample_flow.json`

The `sample_flow.json` file provides an example of a flow definition in JSON format. It demonstrates how to structure a flow using the different block types and their properties.

## `flow_template.json`

The `flow_template.json` file provides a template for defining flows in JSON format. It serves as a starting point for creating new flow definitions.

## `scheduling_logic.json`

The `scheduling_logic.json` file defines the processing logic for scheduling-related blocks. It specifies the inputs, outputs, and dependencies of each block involved in the scheduling process.

## `block_library.json`

The `block_library.json` file defines a library of reusable blocks that can be used in flows. It provides a collection of predefined blocks with their properties, methods, and API integration details.

## `flowgorithm.rs`

The `flowgorithm.rs` file defines the `Flowgorithm` struct and its methods for processing user instructions, generating logic, and creating flows.

## Flow Execution

The flow execution process begins by loading a flow definition from a JSON file using the `load_flow` method of the `FlowEngine`. The loaded flow is then stored in the `flow` field of the `FlowEngine` struct.

To execute the flow, the `execute` method is called on the `FlowEngine` instance. The execution starts from the initial block specified by the `start_block_id` property in the flow definition.

The `execute` method iteratively processes each block in the flow using the `process_block` method. The `process_block` method determines the type of the block and calls the corresponding `process_*` method based on the block type.

## API Integration

The flows crate supports API integration for blocks that require external data or services. The `ApiIntegration` struct defines the configuration for API integration, including the API URL, request format, response format, response status, and authentication.

## Weighted Flows

The flow execution logic in `flows.rs` allows for probabilistic decision-making through weighted flows. Each block can have fixed connections or weighted connections, and the next block is selected based on the calculated weights.

## Conclusion

The flows crate provides a flexible and extensible framework for defining and executing flows. It allows you to create complex flows using a declarative JSON format, specifying the blocks, their properties, connections, and weights.

By leveraging the flows crate, you can create interactive and dynamic applications that require flow-based logic, including user input, decision-making, conditional execution, and external data retrieval.