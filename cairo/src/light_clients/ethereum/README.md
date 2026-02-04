# Ethereum Light Client (BankaiOS standalone)

This module ports the existing Ethereum light client logic into a self‑contained package that can be run without BankaiOS. The entrypoint is `run(prev: EthereumClientOutput, network_id: felt) -> EthereumClientOutput`, and all witness data is loaded inside that entrypoint via the existing hints. Network‑specific config (fork schedule, domains, genesis root) is selected by `network_id`.

## Entrypoint

- `run(prev: EthereumClientOutput, network_id: felt) -> EthereumClientOutput`
  - Loads `EthereumConfig` for the given network (sync committee period, genesis root, fork schedule).
  - Reads witness inputs using the same hints as the current circuit:
    - `write_consensus_inputs()` (sets `consensus_inputs`, `is_genesis`, `is_committee_update`, `program_hash`)
    - `write_committee_update_inputs()` (only when committee update is requested)
  - Executes the beacon and execution updates and returns the updated `EthereumClientOutput`.

## Inputs and Data Flow

1. **Previous state**: `prev` provides the prior beacon/execution outputs needed for continuity.
   - If `is_genesis == 1`, the circuit replaces `prev` with `get_ethereum_genesis(config.genesis_validator_root)`.
2. **Consensus inputs**: `ConsensusInputs` are loaded inside the entrypoint from hints.
3. **Beacon update**:
   - SSZ header hash
   - Signing root computation (network fork schedule + fork domains)
   - BLS signature verification
   - Beacon MMR update
4. **Execution update**:
   - Execution payload header hash
   - SSZ inclusion proof against the beacon body root
   - Execution MMR update
5. **Committee update (optional)**:
   - If `is_committee_update == 1`, run committee update and replace `next_validator_root`.

## Assumptions

- **Network selection**: `network_id` is expected to be one of `Networks.MAINNET` or `Networks.SEPOLIA` (see `config/`).
- **Sync committee transitions**: transition boundary is computed from slots via `config.sync_committee_period`.
- **Validator roots**:
  - On transition, `previous.beacon.next_validator_root` must match the input validator root.
  - Otherwise, `previous.beacon.current_validator_root` must match the input validator root.
- **MMR updates**: `run_beacon_mmr_update` and `run_execution_mmr_update` are used as‑is (same hints / witness data).
- **Fork domains**: `Fork.get_id` uses a hint with the fork schedule and is range‑checked; domains are looked up from precomputed per‑network tables.
- **Genesis**: uses `config.genesis_validator_root` for the selected network.

## Outputs

`EthereumClientOutput` contains:

- `beacon: BeaconClientOutput`
  - slot, header/state roots, justification/finalization heights
  - signer count, MMR roots, validator roots
- `execution: ExecutionClientOutput`
  - execution block number, header hash
  - justification/finalization heights
  - MMR roots

The output matches the semantics of the original Cairo light client; only the surrounding orchestration has been relocated into this module.
