---
id: design
title: Internal Design
sidebar_label: Internal Design
---

import useBaseUrl from '@docusaurus/useBaseUrl';

## Goals

1. (Non-repudiation) Proofs of game state cannot be unilaterally created by a single player
2. (Reproducibility) Proofs must always be serializable
3. (Succinctness) Proofs do not require full game history to validate game state
4. (Non-redundancy) Proofs do not require more than one attestation per player
5. (Non-serializability) Non-serializable game states have a representation
6. (Unbiased randomness) Randomness should be fair and collectively decided by the players

The succinctness and non-redundancy properties are important for minimizing proof sizes and state computation.
The non-serializability property is important for situations where the current game state is merely an intermediate step towards computing some finalized successor state.
Unbiased randomness is important for ensuring fair gameplay.

## Architecture

Any game with non-serializable states can be transformed into an equivalent transition system of serializable states, assuming the initial state is serializable.
This is because every state can be reproduced from an initial state and a sequence of transitions, all of which are serializable.
In cases where the state is already serializable, the sequence is empty.
In the design, proofs are analogous to game states and diffs are analogous to game actions, and diffs are applied to proofs just as game actions are applied to game states.

<img alt="Transition system equivalence" src={useBaseUrl('img/transform.svg')} />

Succinctness is achieved by pruning game history from the most recent serializable state attested to by all players.
Non-redundancy is achieved by requiring players to sign any new proofs they create via a state transition.
The corresponding diff itself is also signed to prevent denial-of-service attacks and out-of-sequence messages, and to detect when players create invalid state transitions.

Fair randomness is achieved by commit-reveal.
The first player broadcasts a hash commitment to a seed for a random number generator.
The second player broadcasts their seed.
Finally, the first player broadcasts the preimage to their commitment.
The two seeds are combined in order to seed a common random number generator for computing the next state.

This is implemented in Rust using futures.
Futures allow us to leverage the language's async-await to allow the state transition function to be suspended in a non-serializable state while the commit-reveal resolves.
In general, it allows developers to write code that appears synchronous despite executing asynchronously.
For Arcadeum, this gives developers the ability to inline requests for randomness and secret state directly into game logic code without having to manually handle asynchronous interactions such as commit-reveal.

## Implementation

A game state proof is:

* A serializable head state
* A sequence of player actions
* A collection of player proofs, one for each player

A player proof is:

* A range identifying a subsequence of the player actions
* A player's signature of the serialized state and actions identified by the range

A diff is:

* A sequence of actions being appended by a player
* A player's signature of the *updated* serialized state and actions identified by the *updated* range
* A player's signature of the diff itself

### Diff application

Applying a diff to a proof involves:

* Appending the diff's actions to the proof's actions
* Verifying the diff and proof signatures
* Verifying each state transition
* Updating the diff author's proof accordingly
* Pruning actions that are no longer needed for satisfying non-repudiation

<img alt="Diff application" src={useBaseUrl('img/apply.svg')} />

1. The proof has a serializable head state that has been signed by the blue player.
   The orange player has appended their game actions.
   None of the resulting states are serializable, so the orange player's signature covers the full range of the proof.
2. The blue player appends their actions.
   There's a serializable intermediate state.
   The blue player updates their signature to cover only that state and the actions following it.
3. The orange player appends their actions.
   They update the game state to a serializable one and sign only that state.
4. Actions prior to the blue player's head state are no longer needed and therefore pruned.
   They can be stored for archival purposes, but they're no longer an important part of the non-repudiation requirement.

This forms the foundation of the proving layer.
This layer is sufficient for verifying outcomes of games.

### Store layer

Just above the proving layer is the store layer.
It provides a custom state wrapper that automatically handles commit-reveal randomness and secret revealing, and provides an interface for logging game events.
It also allows clients to register hooks for handling automatic dispatch of actions.

<img alt="Store state wrapper" src={useBaseUrl('img/store.svg')} />
