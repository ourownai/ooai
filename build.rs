fn main() {
    println!("cargo:rerun-if-changed=src/protos/my_grpc_service.proto");
    tonic_build::configure()
        .out_dir("src/protos")
        .compile(&["src/protos/my_grpc_service.proto"], &["src/protos"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));
}
