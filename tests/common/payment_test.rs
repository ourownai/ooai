#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffers::payment_buffer::PaymentBuffer;
    use crate::iam::merkle_tree::MerkleTree;
    use std::collections::HashMap;

    #[test]
    fn test_payment_processing_with_merkle_tree() {
        // Create payment providers
        let mut config = HashMap::new();
        config.insert("base_url".to_string(), "https://rest.payment-provider.com".to_string());
        let rest_provider = PaymentProviderFactory::create_payment_provider("rest", config).unwrap();

        config.clear();
        config.insert("endpoint".to_string(), "grpc.payment-provider.com:1234".to_string());
        let grpc_provider = PaymentProviderFactory::create_payment_provider("grpc", config).unwrap();

        let webhooks_provider = PaymentProviderFactory::create_payment_provider("webhooks", HashMap::new()).unwrap();

        config.clear();
        config.insert("mint_address".to_string(), "OAI_MINT_ADDRESS".to_string());
        let solana_provider = PaymentProviderFactory::create_payment_provider("solana", config).unwrap();

        // Create payment processor
        let mut payment_providers = HashMap::new();
        payment_providers.insert("rest".to_string(), rest_provider);
        payment_providers.insert("grpc".to_string(), grpc_provider);
        payment_providers.insert("webhooks".to_string(), webhooks_provider);
        payment_providers.insert("solana".to_string(), solana_provider);
        let payment_processor = PaymentProcessor::new(payment_providers);

        // Create payment buffer
        let mut payment_buffer = PaymentBuffer::new(100);

        // Process payments and add to buffer
        let rest_result = payment_processor.charge_card("rest", "card_token_1", 100.0);
        assert!(rest_result.is_ok());
        let rest_transaction_id = rest_result.unwrap();
        payment_buffer.add_payment(rest_transaction_id, 100.0);

        let grpc_result = payment_processor.charge_card("grpc", "card_token_2", 50.0);
        assert!(grpc_result.is_ok());
        let grpc_transaction_id = grpc_result.unwrap();
        payment_buffer.add_payment(grpc_transaction_id, 50.0);

        let solana_result = payment_processor.charge_card("solana", "card_token_3", 75.0);
        assert!(solana_result.is_ok());
        let solana_transaction_id = solana_result.unwrap();
        payment_buffer.add_payment(solana_transaction_id, 75.0);

        // Create Merkle tree from payment buffer
        let merkle_tree = MerkleTree::new(payment_buffer.get_payments());

        // Verify payment inclusion in Merkle tree
        let rest_proof = merkle_tree.generate_proof(0);
        assert!(merkle_tree.verify_proof(&rest_proof, 0, &merkle_tree.root()));

        let grpc_proof = merkle_tree.generate_proof(1);
        assert!(merkle_tree.verify_proof(&grpc_proof, 1, &merkle_tree.root()));

        let solana_proof = merkle_tree.generate_proof(2);
        assert!(merkle_tree.verify_proof(&solana_proof, 2, &merkle_tree.root()));
    }
}
