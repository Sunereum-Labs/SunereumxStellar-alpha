# Sunereum x Soroban - explanatory architecture

# Sunereum Labs Technical Architecture

## Overview

Sunereum Labs is developing a cutting-edge blockchain-based insurance platform for clean energy assets, leveraging the Stellar network and Soroban smart contracts. This README provides an overview of our technical architecture, focusing on the integration of our existing systems with the Stellar blockchain.

## Insurance Policy Management Workflow

When a client has an insurance policy in our ReSpark platform, the following workflow is triggered:

1. Policy Creation:
   - The policy is created in the ReSpark platform with all necessary details.
   - A corresponding smart contract is deployed on the Stellar network using Soroban.

2. Whitelisting:
   - The Sunereum entity is whitelisted in the smart contract, granting it the right to trigger actions on behalf of the client.

3. Smart Contract Interaction:
   - The whitelisted Sunereum entity can now interact with the smart contract to:
     - Update policy details
     - Process claims
     - Trigger risk assessments

## Smart Contract Structures

### Insurance Policy Smart Contract

Key fields represented on-chain:
- `policyType`: String (e.g., "SolarAllRisk")
- `duration`: Uint64 (in seconds)
- `monthlyPremium`: Int128 (in USDC, with 7 decimal places)
- `policyHolder`: Address
- `whitelistedEntity`: Address (Sunereum's address)
- `isActive`: Bool

### Risk Category Smart Contracts

Each insurance contract is associated with three key risk category smart contracts:

1. Climate Risk Smart Contract
2. Technology Risk Smart Contract
3. Operational Risk Smart Contract

## Platform Integrations

### Technology Risk - Virya IoT Platform Integration

The Technology Risk smart contract interfaces with Sunereum's IoT Virya platform:

1. Virya Platform monitors the client's system status.
2. If the system is non-operational for over 4 hours:
   - Virya Platform sends a payload to the Technology Risk smart contract.
   - The smart contract updates the risk status and may trigger predefined actions (e.g., notify stakeholders, initiate claims process).

### Operational Risk - Maintenance Log Integration

The Operational Risk smart contract manages maintenance records:

1. When a maintenance report is submitted in the ReSpark platform:
   - The Operational Risk smart contract is called to update the maintenance log.
   - The smart contract stores a hash of the report and timestamp on-chain.
   - Full report details are stored off-chain with the hash serving as a reference.

### Climate Risk - Sunereum Climate Risk Engine Integration

The Climate Risk smart contract interfaces with Sunereum's climate risk engine:

1. The climate risk engine continuously analyzes weather data and patterns.
2. The engine updates the Climate Risk smart contract with the site's risk status:
   - `LOW_RISK`
   - `MEDIUM_RISK`
   - `HIGH_RISK`
3. This update occurs daily or when significant changes in risk are detected.
4. The smart contract can trigger predefined actions based on risk levels (e.g., send warnings, adjust premiums).

## Data Flow

1. ReSpark Platform ⇄ Stellar Network
   - Policy creation and updates
   - Claims processing initiation

2. Virya IoT Platform → Technology Risk Smart Contract
   - System operational status updates

3. ReSpark Platform → Operational Risk Smart Contract
   - Maintenance log updates

4. Climate Risk Engine → Climate Risk Smart Contract
   - Daily risk status updates

5. Smart Contracts ⇄ Sunereum Whitelisted Entity
   - Risk assessments
   - Policy adjustments
   - Claims processing

This architecture ensures a seamless integration between Sunereum's existing platforms (ReSpark and Virya) and the Stellar blockchain, enabling efficient, transparent, and automated insurance operations for clean energy assets.
