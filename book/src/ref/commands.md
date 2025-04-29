# Commands

Starkli offers the following commands:

## selector
Calculate selector from name

**Usage:** starkli selector `<NAME>`

**Arguments:**
  `<NAME>`  Selector name

**Options:**
`-h, --help`  Print help

---

## class-hash
Calculate class hash from any contract artifacts (Sierra, casm, legacy)

**Usage:** starkli class-hash `<FILE>`

**Arguments:**
  `<FILE>`  Path to contract artifact file

**Options:**
  `-h, --help`  Print help

---

## to-cairo-string
Encode string into felt with the Cairo short string representation

**Usage:** starkli to-cairo-string [OPTIONS] `<TEXT>`

**Arguments:**
  `<TEXT>`  Text to be encoded in felt

**Options:**
  
  `--dec`   Display the encoded value in decimal representation 
  
  `-h, --help`  Print help

---

## parse-cairo-string
Decode string from felt with the Cairo short string representation

**Usage:** starkli parse-cairo-string `<FELT>`

**Arguments:**
  `<FELT>`  Encoded string value in felt, in decimal or hexadecimal representation

**Options:**
  `-h, --help`  Print help

---

## mont
Print the montgomery representation of a field element

**Usage**: starkli mont [OPTIONS] `<FELT>`

**Arguments:**
  `<FELT>`  Encoded string value in felt, in decimal or hexadecimal representation

**Options:**

`--hex`   Emit array elements in hexadecimal format

`-h, --help`  Print help

---

## call
Call contract functions without sending transactions

**Usage:** starkli call [OPTIONS] `<CONTRACT_ADDRESS>` `<SELECTOR>` `[CALLDATA]...`

**Arguments:**

  `<CONTRACT_ADDRESS>`  Contract address
  
  `<SELECTOR>`          Name of the function being called
  
  `[CALLDATA]...`       Raw function call arguments

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---

## transaction
Get Starknet transaction by hash

**Usage:** starkli transaction [OPTIONS] `<HASH>`

**Arguments:**
`<HASH>`  Transaction hash

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

  
---

## block-number
Get latest block number

**Usage:** starkli block-number [OPTIONS]

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help
  
---

## block-hash
Get latest block hash

**Usage:** starkli block-hash [OPTIONS]

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---

## block
Get Starknet block

**Usage:** starkli block [OPTIONS] [BLOCK_ID]

**Arguments:**
  `[BLOCK_ID]`  Block number, hash, or tag (latest/pending) [default: latest]

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--full`               Fetch full transactions instead of hashes only

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help
  
---

## block-time
Get Starknet block timestamp only

**Usage:** starkli block-time [OPTIONS] [BLOCK_ID]

**Arguments:**
  `[BLOCK_ID]`  Block number, hash, or tag (latest/pending) [default: latest]

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--unix`               Show block time in Unix timestamp format

`--rfc2822            Show block time in RFC 2822 format

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---

## state-update
Get state update from a certain block

**Usage:** starkli state-update [OPTIONS] [BLOCK_ID]

**Arguments:**
  `[BLOCK_ID]`  Block number, hash, or tag (latest/pending) [default: latest]

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---

## transaction-receipt
Get transaction receipt by hash

**Usage:** starkli transaction-receipt [OPTIONS] `<HASH>`

**Arguments:**
  `<HASH>`  Transaction hash

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---

## chain-id
Get Starknet network ID

**Usage:** starkli chain-id [OPTIONS] [BLOCK_ID]

**Arguments:**
  `[BLOCK_ID]`  Block number, hash, or tag (latest/pending) [default: latest]

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--no-decode`          Do not show the decoded text

`--dec`                Display the decimal instead of hexadecimal representation

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

## balance

TBD

---

## nonce
Get nonce for a certain contract

**Usage:** starkli nonce [OPTIONS] `<ADDRESS>`

**Arguments:**
  `<ADDRESS>`  Contract address

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`h, --help`               Print help

---

## storage
Get storage value for a slot at a contract

**Usage:** starkli storage [OPTIONS] `<ADDRESS>` `<KEY>`

**Arguments:**

  `<ADDRESS`  Contract address
  
  `<KEY>`      Storage key

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help `              Print help

---

## class-hash-at
Get contract class hash deployed at a certain address

**Usage:** starkli class-hash-at [OPTIONS] `<ADDRESS>`

**Arguments:**
  `<ADDRESS>`  Contract address

**Options:**

`-rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---

## class-by-hash
Get contract class by hash

**Usage:** starkli class-by-hash [OPTIONS] `<HASH>`

**Arguments:**
  `<HASH>`  Class hash

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---
  
## class-at
Get contract class deployed at a certain address

**Usage:** starkli class-at [OPTIONS] <ADDRESS>

**Arguments:**
  `<ADDRESS>`  Contract address

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---

## syncing
Get node syncing status

**Usage:** starkli syncing [OPTIONS]

**Options:**

`--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--log-traffic`        Log raw request/response traffic of providers

`-h, --help`               Print help

---
## signer
Signer management commands

**Usage:** starkli signer `<COMMAND>`

**Commands:**
- **`keystore`**     Keystore management commands

    **Usage:** starkli signer keystore `<COMMAND>`

    **Commands:**
    - **`new`**              Randomly generate a new keystore
    
        **Usage:** starkli signer keystore new [OPTIONS] `<FILE>`
        
        **Arguments:**
        `<FILE>`  Path to save the JSON keystore
        
        **Options:**
        
        `--password <PASSWORD>`  Supply password from command line option instead of prompt
        
        `-force`                Overwrite the file if it already exists
        
        `-h, --help`                 Print help
        
    - **`from-key`**         Create a keystore file from an existing private key
    
        **Usage:** starkli signer keystore from-key [OPTIONS] `<FILE>`
        
        **Arguments:**
        `<FILE>`  Path to save the JSON keystore
        
        **Options:**
        
        `--force`                Overwrite the file if it already exists
        
        `--private-key-stdin`    Take the private key from stdin instead of prompt
        
        `--password <PASSWORD>`  Supply password from command line option instead of prompt
        
        `-h, --help`                 Print help

    - **`inspect`**          Check the public key of an existing keystore file

        **Usage:** starkli signer keystore inspect [OPTIONS] `<FILE>`
        
        **Arguments:**
        `<FILE>`  Path to the JSON keystore
        
        **Options:**
        
        `--password <PASSWORD>`  Supply password from command line option instead of prompt
        
        `--raw`                  Print the public key only
        
        `-h, --help`                 Print help
        
    - **`inspect-private`**  Check the private key of an existing keystore file
        **Usage:** starkli signer keystore inspect-private [OPTIONS] `<FILE>`
        
        **Arguments:**
        `<FILE>`  Path to the JSON keystore
        
        **Options:**
    
        `--password <PASSWORD>`  Supply password from command line option instead of prompt
        
        `--raw`                  Print the private key only
        
        `-h, --help`                 Print help
  
    - **`help`**             Print this message or the help of the given subcommand(s)
    
    **Options:**
      `-h, --help`  Print help
  
- **`gen-keypair`**  Randomly generate a new key pair
    **Usage:** starkli signer gen-keypair
    
    **Options:**
      `-h, --help`  Print help

**Options:**
    `-h, --help`  Print help

---

## account

Account management commands

**Usage:** starkli account `<COMMAND>`

**Commands:**
- **`fetch`** Fetch account config from an already deployed account contract

    **Usage:** starkli account fetch [OPTIONS] `<ADDRESS>`
    
    **Arguments:** `<ADDRESS>`  Contract address
    
    **Options:**
    
    `--rpc <RPC>`          Starknet JSON-RPC endpoint [env: STARKNET_RPC=]
  
    `--network <NETWORK>`  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]
    
    `--force`              Overwrite the file if it already exists
    
    `--output <OUTPUT>`    Path to save the account config file
    
    `--log-traffic`        Log raw request/response traffic of providers
    
    `-h, --help`               Print help

- **`deploy`** Deploy account contract with a DeployAccount transaction
    
    **Usage:** starkli account deploy [OPTIONS] <FILE>
    
    **Arguments:** `<FILE>`  Path to the account config file
    
    **Options:**

    `--rpc <RPC>`
    Starknet JSON-RPC endpoint [env: STARKNET_RPC=]
    
    `--network <NETWORK>`
    Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]
    
    `--keystore <KEYSTORE>`
    Path to keystore JSON file [env: STARKNET_KEYSTORE=/Users/danielbejarano/Dev/starkli-wallets/deployer/keystore.json]
    
    `--keystore-password <KEYSTORE_PASSWORD>`
    Supply keystore password from the command line option instead of prompt
    
    `--private-key <PRIVATE_KEY>`
    Private key in hex in plain text
    
    `--max-fee <MAX_FEE>`
    Maximum transaction fee in Ether (18 decimals)
    
    `--max-fee-raw <MAX_FEE_RAW>`
    Maximum transaction fee in Wei
    
    `--estimate-only`
    Only estimate transaction fee without sending the transaction
    
    `--log-traffic`
    Log raw request/response traffic of providers
    
    `-h, --help`
    Print help
- **`oz`** Create and manage OpenZeppelin account contracts

    **Usage:** starkli account oz `<COMMAND>`

    **Commands:**
    
    - **`init`**  Create a new account configuration without actually deploying

        **Usage:** starkli account oz init [OPTIONS] `<OUTPUT>`

        **Arguments:**
        `<OUTPUT>`  Path to save the account config file

        **Options:**
    
        `--keystore <KEYSTORE>`
              Path to keystore JSON file [env: STARKNET_KEYSTORE=]
             
         
        `--keystore-password <KEYSTORE_PASSWORD>`
              Supply keystore password from command line option instead of prompt
              
        `--private-key <PRIVATE_KEY>`
              Private key in hex in plain text
              
        `-f, --force`
              Overwrite the account config file if it already exists
              
        `-h, --help`
              Print help

    - **`help`**  Print this message or the help of the given subcommand(s)
    

**Options:**
`-h, --help`  Print help

---

## invoke
Send an invoke transaction from an account contract

**Usage:** starkli invoke [OPTIONS] `--account <ACCOUNT>` `[CALLS]...`

**Arguments:**
  `[CALLS]...`  One or more contract calls. See documentation for more details

**Options:**

`--rpc <RPC>`
  Starknet JSON-RPC endpoint [env: STARKNET_RPC=]
  
`--network <NETWORK>`
  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]
  
`--keystore <KEYSTORE>`
  Path to keystore JSON file [env: STARKNET_KEYSTORE=]
  
`--keystore-password <KEYSTORE_PASSWORD>`
  Supply keystore password from command line option instead of prompt
  
`--private-key <PRIVATE_KEY>`
  Private key in hex in plain text
  
`--account <ACCOUNT>`
  Path to account config JSON file [env: STARKNET_ACCOUNT=]
  
`--max-fee <MAX_FEE>`
  Maximum transaction fee in Ether (18 decimals)
  
`--max-fee-raw <MAX_FEE_RAW>`
  Maximum transaction fee in Wei
  
`--estimate-only`
  Only estimate transaction fee without sending transaction
  
`--watch`
  Wait for the transaction to confirm
  
`--log-traffic`
  Log raw request/response traffic of providers
  
`-h, --help`
  Print help

---

## declare
Declare a contract class

**Usage:** starkli declare [OPTIONS] `--account <ACCOUNT>` `<FILE>`

**Arguments:**
  `<FILE>`  Path to contract artifact file

**Options:**

`--rpc <RPC>`
Starknet JSON-RPC endpoint [env: STARKNET_RPC=]

`--network <NETWORK>`
Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]

`--keystore <KEYSTORE>`
Path to keystore JSON file [env: STARKNET_KEYSTORE=]

`--keystore-password <KEYSTORE_PASSWORD>`
Supply keystore password from command line option instead of prompt

`--private-key <PRIVATE_KEY>`
Private key in hex in plain text

`--compiler-version <COMPILER_VERSION>`
Statically-linked Sierra compiler version [possible values: 2.0.1, 2.1.0]

`--casm-hash <CASM_HASH>`
Override Sierra compilation and use CASM hash directly

`--account <ACCOUNT>`
Path to account config JSON file [env: STARKNET_ACCOUNT=]

`--max-fee <MAX_FEE>`
Maximum transaction fee in Ether (18 decimals)

`--max-fee-raw <MAX_FEE_RAW>`
Maximum transaction fee in Wei

`--estimate-only`
Only estimate transaction fee without sending transaction

`--watch`
Wait for the transaction to confirm

`--log-traffic`
Log raw request/response traffic of providers

`-h, --help`
Print help

---

## deploy
Deploy contract via the Universal Deployer Contract

**Usage:** starkli deploy [OPTIONS] `--account <ACCOUNT>` `<CLASS_HASH>` `[CTOR_ARGS]...`

**Arguments:**

`<CLASS_HASH`    Class hash

`[CTOR_ARGS]...`  Raw constructor arguments

**Options:**

`--rpc <RPC>`
  Starknet JSON-RPC endpoint [env: STARKNET_RPC=]
  
`--network <NETWORK>`
  Starknet network [env: STARKNET_NETWORK=] [possible values: mainnet, goerli-1, goerli-2, integration]
  
`--keystore <KEYSTORE>`
  Path to keystore JSON file [env: STARKNET_KEYSTORE=]
  
`--keystore-password <KEYSTORE_PASSWORD>`
  Supply keystore password from command line option instead of prompt
  
`--private-key <PRIVATE_KEY>`
  Private key in hex in plain text
  
`--not-unique`
  Do not derive contract address from deployer address
  
`--account <ACCOUNT>`
  Path to account config JSON file [env: STARKNET_ACCOUNT=]
  
`--max-fee <MAX_FEE>`
  Maximum transaction fee in Ether (18 decimals)
  
`--max-fee-raw <MAX_FEE_RAW>`
  Maximum transaction fee in Wei
  
`--estimate-only`
  Only estimate transaction fee without sending transaction
  
`--salt <SALT>`
  Use the given salt to compute contract deploy address
  
`--watch`
  Wait for the transaction to confirm
  
`--log-traffic`
  Log raw request/response traffic of providers
  
`-h, --help`
  Print help

---


## completions
Generate shell completions script

**Usage:** starkli completions `<SHELL>`

**Arguments:**
  `<SHELL>`  Shell name [possible values: bash, elvish, fish, powershell, zsh]

**Options:**
  `-h, --help`  Print help
