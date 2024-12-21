# build only the wasm contracts, avoid building the workspace-test program
export RUSTFLAGS='-C link-arg=-s'
cargo build -p gradual-release-claim-contract --target wasm32-unknown-unknown --release
cargo build -p nep141-test-token --target wasm32-unknown-unknown --release
# copy to res folder
set -ex
mkdir -p res
cp target/wasm32-unknown-unknown/release/gradual_release_claim_contract.wasm res/
cp target/wasm32-unknown-unknown/release/nep141_test_token.wasm res/
set +ex
# warn about rustc version
echo =========================================================
echo RUST version:
rustc --version
echo "WARN: because we're using near-sdk 4.0.0, we need to use rustc 1.81.0"
echo "WARN: If rustc version is 1.82 or 1.83 after deploy in the blockchain"
echo "WARN: you will get Deserialization ERROR: wasm execution failed with error:"
echo "WARN: CompilationError(PrepareError(Deserialization))"
echo "========================================================="
