IAM (Identity and Access Management) Crate
The IAM crate provides a set of modules and functionality for managing user identities, access control, and authentication using Keycloak as the identity provider. It also includes support for verifiable credentials and digital wallets.

Modules
group.rs
The group.rs module defines the Group struct, which represents a group of users. It provides methods for creating, modifying, and managing groups, such as adding and removing users, setting group descriptions, and finding users within a group.

iam.rs
The iam.rs module serves as the main entry point for the IAM functionality. It includes functions for creating realms, users, and interacting with Keycloak using the KeycloakController. It also provides error handling and defines the KeycloakError enum for IAM-related errors.

jwt.rs
The jwt.rs module handles JSON Web Tokens (JWTs) and provides functionality for encoding and decoding tokens. It defines the JWT struct and includes methods for adding and retrieving payload data, as well as encoding and decoding JWTs using the jsonwebtoken library.

keycloak_provider.rs
The keycloak_provider.rs module provides integration with Keycloak, an open-source identity and access management solution. It includes functions for creating realms, users, and interacting with the Keycloak API using the reqwest library.

proofs.rs
The proofs.rs module defines the Proof struct, which represents a cryptographic proof associated with a verifiable credential. It includes fields for the proof type, creation timestamp, verification method, and the associated JWT.

user.rs
The user.rs module defines the User struct, which represents a user in the system. It includes fields for the user's ID, public key, and secret key. The module provides a constructor function for creating new user instances.

verifiable_credentials.rs
The verifiable_credentials.rs module defines the VerifiableCredential struct, which represents a verifiable credential. It includes fields for the credential subject, issuer, issuance date, expiration date, and associated proof. The module provides functionality for creating and managing verifiable credentials.

wallet.rs
The wallet.rs module defines the Wallet struct, which represents a digital wallet for storing and managing verifiable credentials. It includes fields for the owner's DID (Decentralized Identifier), encrypted DID document, encrypted verifiable credentials, signing keys, and wallet address. The module provides methods for creating new wallets, storing and retrieving verifiable credentials, and signing and verifying data using the wallet's keys.

Usage
To use the IAM crate in your Rust project, add the following dependency to your Cargo.toml file:

[dependencies]
iam = { path = "path/to/iam" }



Then, you can use the various modules and functionality provided by the IAM crate in your code. For example:

use iam::iam::KeycloakController;
use iam::user::User;
use iam::wallet::Wallet;

// Create a new Keycloak controller
let keycloak_controller = KeycloakController::new(/* ... */);

// Create a new user
let user = User::new(/* ... */);

// Create a new wallet
let wallet = Wallet::new_wallet();

// Perform IAM operations using the modules and functions provided by the crate



Make sure to configure the necessary environment variables or provide the required parameters when initializing the KeycloakController and other components.

Contributing
Contributions to the IAM crate are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request on the GitHub repository.

License
The IAM crate is open-source software licensed under the AGPLv3 license.