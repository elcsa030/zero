# Bonsai Ethereum Relay
This repository provides the `bonsai-ethereum-relay`, a tool to integrate Ethereum with Bonsai. It is coupled with an Ethereum Smart Contract able to proxy the interaction from Ethereum to Bonsai and vice versa.

## Usage
```console
Usage: bonsai-ethereum-relay [OPTIONS] --contract-address <CONTRACT_ADDRESS> --eth-node-url <ETH_NODE_URL> --wallet-key-identifier <WALLET_KEY_IDENTIFIER>

Options:
  -p, --port <PORT>
          The port of the relay server API [default: 8080]
      --publish-mode
          Toggle to disable the relay server API
      --contract-address <CONTRACT_ADDRESS>
          Bonsai Relay contract address on Ethereum
      --eth-node-url <ETH_NODE_URL>
          Ethereum Node endpoint
      --eth-chain-id <ETH_CHAIN_ID>
          Ethereum chain ID [default: 5]
  -w, --wallet-key-identifier <WALLET_KEY_IDENTIFIER>
          Wallet Key Identifier. Can be a private key as a hex string, or an AWS KMS key identifier [env: WALLET_KEY_IDENTIFIER]
      --use-kms
          Toggle to use a KMS client
  -h, --help
          Print help
  -V, --version
          Print version
```

A typical flow works as follows:
1. Deploy a Bonsai Relay Smart Contract on Ethereum at a given address `0xB..`.
2. Start an instance of the relay tool configured with the option `--contract-address` defined as `0xB..`.
3. Delegate some off-chain computation for a given Smart Contract `A` to Bonsai by registering the `Image` or `ELF` (i.e., the compiled binary responsible for executing the given computation on the RISC Zero ZKVM) to Bonsai.
4. The corresponding `Image ID` and the Bonsai Relay Smart Contract `0xB..` can be used to construct and deploy the Smart Contract `A` to Ethereum.
5. Send a transaction to Smart Contract `A` to trigger a `Callback request` event that the Bonsai Relay will catch and forward to Bonsai.
6. Once Bonsai has generated a proof of execution, the Bonsai Relay will forward this proof along with the result of the computation to the Bonsai Relay Smart Contract.
7. This triggers a verification of the proof on-chain, and only upon successful verification, the result of the computation will be forwarded to the original requester Smart Contract `A`.

### Publish mode
As an alternative to trigger a `Callback request` from Ethereum as described by step 5, the request can be sent directly to the Bonsai Relay via an HTTP REST API. Then, the remaining steps will flow as above. The following example explains how to do that.

#### Example Usage
The following example assumes that the Bonsai Relay is up and running with the server API enabled,
and that the memory image of your `ELF` is already registered against Bonsai with a given `IMAGE_ID` as its identifier.

```rust
// initialize a relay client
let relay_client = Client::from_parts(
        "http://localhost:8080".to_string(), // here goes the actual url of the Bonsai Relay
        "BONSAI_API_KEY" // here goes the actual Bonsai API-Key
    )
    .expect("Failed to initialize the relay client");

// Initialize the input for the guest.
// In this example we are sending a slice of bytes,
// where the first 4 bytes represents the length
// of the slice (in little endian).
let mut input = vec![0; 36];
input[0] = 32;
input[35] = 100;

// Create a CallbackRequest for the your contract
// example: (tests/solidity/contracts/Counter.sol).
let image_id: [u8; 32] = bytemuck::cast(IMAGE_ID);
let request = CallbackRequest {
    callback_contract: counter.address(),
    // you can use the command `solc --hashes tests/solidity/contracts/Counter.sol`
    // to get the value for your actual contract
    function_selector: [0xff, 0x58, 0x5c, 0xaf],
    gas_limit: 3000000,
    image_id,
    input,
};

// Send the callback request to the Bonsai Relay.
relay_client
    .callback_request(request)
    .await
    .expect("Callback request failed");

```
