# Project Management Smart Contract

This smart contract facilitates the management of project tasks, contributions, and rewards on the Solana blockchain.

## Workflow

1. **Task Creation**: The project owner creates a task using the `add_task` instruction, specifying the task details such as ID, title, description, status, and reward breakdown for different stages (concept, testing, production).

2. **Contributor Registration**: Contributors register themselves using the `add_contributor` instruction, providing their ID, name, email, wallet address, and accepting the Individual Contributor License Agreement (ICLA) by specifying the ICLA version and IPFS hash.

3. **ICLA Acceptance**: When a contributor registers, they must accept the ICLA by providing the ICLA version and IPFS hash. The smart contract stores the ICLA acceptance details, including the timestamp of acceptance.

4. **Task Assignment**: The project owner assigns a task to a contributor using the `assign_task` instruction, specifying the task ID and the contributor's wallet address. This updates the task status to "assigned" and associates the contributor with the task.

5. **Contribution Submission**: Contributors work on the assigned tasks and make contributions by creating pull requests on the project's GitHub repository. They add the contribution details using the `add_contribution` instruction, providing the task ID, contributor ID, commit hash, and merge stage.

6. **Contribution Validation**: When a pull request is merged into the testing branch, the `mint_contribution_nft` instruction is called with the contribution ID. The smart contract validates the contribution by checking if the merge stage is "testing", if the contributor and task match the specified IDs, and if the contributor has accepted the ICLA.

7. **NFT Minting**: If the contribution validation passes, the smart contract mints an NFT to the contributor's token account. The NFT metadata includes the task details, contributor information, and the breakdown percentage of the fee for the task.

8. **Reward Distribution**: The rewards for the task are distributed to the contributor based on the NFT metadata and the specified reward breakdown for each stage (concept, testing, production).

## Setup and Deployment

1. Install the necessary dependencies, including Rust, Solana CLI, and Anchor framework.

2. Clone the project repository and navigate to the project directory.

3. Update the `Anchor.toml` file with your desired configuration settings.

4. Build the project using `anchor build`.

5. Deploy the smart contract to the Solana blockchain using `anchor deploy`.

6. Interact with the smart contract using the provided instructions (`add_task`, `add_contributor`, `assign_task`, `add_contribution`, `mint_contribution_nft`) through a client application or the Solana CLI.

## GitHub Integration

The smart contract integrates with GitHub to validate contributions. It requires a GitHub API token to access the repository and validate the existence of commits and pull requests.

Make sure to replace the placeholder values (`owner/repo`) in the `validate_contribution` function with your actual GitHub repository details.

## Error Handling

The smart contract defines the following error codes:

- `InvalidContribution`: Indicates that the specified contribution is invalid.
- `InvalidAssignment`: Indicates that the task assignment is invalid.
- `InvalidMergeStage`: Indicates that the merge stage is invalid for minting an NFT.
- `ICLANotAccepted`: Indicates that the contributor has not accepted the ICLA.

Proper error handling should be implemented in the client application to handle these error cases gracefully.

## License

This project is licensed under the AGPLv3 (LICENSE).
