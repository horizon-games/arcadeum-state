---
id: guide
title: User Guide
sidebar_label: User Guide
---

To define your game logic, you need to implement the `arcadeum::store::State` trait for your game state type.

1. Start by defining your basic primitive associated types: `ID` and `Nonce`.

   There's already a default implementation of `ID` for many types that already implement `serde::Serialize` and `serde::Deserialize`, and any primitive numeric type can be used as a `Nonce`, e.g. `u8`, `i64`, etc.
   Make sure you choose a conservative `Nonce` type that won't overflow for your particular game design.

   ```rust
   use arcadeum::store::State;

   #[derive(Clone)]
   struct MyGameState {
       // ...
   }

   impl State for MyGameState {
       type ID = [u8; 8];
       type Nonce = i32;

       // ...
   }
   ```

2. Define the `Action` associated type.

   `Action`s are things your players can do in order to progress the state of the game.
   `Action`s need to be serializable and deserializable, and they also need to be `Clone`-able.
   Most types that already implement `serde::Serialize` and `serde::Deserialize` will automatically be `Action`s.

   ```rust
   use serde::{Deserialize, Serialize};

   // ...

   impl State for MyGameState {
       // ...

       type Action = MyGameAction;

       // ...
   }

   #[derive(Deserialize, Serialize, Clone)]
   struct MyGameAction {
       // ...
   }
   ```

3. Define the `Secret` associated type.

   The `Secret` type defines the structure of the secret data for a player.
   `Secret`s need to be serializable and deserializable, and they also need to be `Clone`-able.
   Most types that already implement `serde::Serialize` and `serde::Deserialize` will automatically be `Secret`s.
   If you're designing a game of perfect information, set this to `()`.

   ```rust
   use serde::{Deserialize, Serialize};

   // ...

   impl State for MyGameState {
       // ...

       type Secret = MyGameSecret;

       // ...
   }

   #[derive(Deserialize, Serialize, Clone)]
   struct MyGameSecret {
       // ...
   }
   ```

4. Implement the `verify` method.

   You need to specify for any given game state if an `Action` for a given player is valid or not.
   Return an `Err` with a `String` error message if invalid, otherwise return an `Ok(())`.

   ```rust
   use arcadeum::Player;

   // ...

   impl State for MyGameState {
       // ...

       fn verify(&self, player: Option<crate::Player>, action: &Self::Action) -> Result<(), String> {
           if player != Some(self.current_player) {
               return Err("not your turn!");
           }

           // more validation...

           Ok(())
       }

       // ...
   }
   ```

5. Implement the `apply` method.

   Your `apply` method will only be called with arguments that pass the `verify` method.
   You'll also receive a *context* argument that can be used to do things like log game state events, generate peer-to-peer randomness, etc.

   ```rust
   use arcadeum::store::Context;

   // ...

   impl State for MyGameState {
       // ...

       fn apply(
           self,
           player: Option<Player>,
           action: &Self::Action,
           context: Context<Self>,
       ) -> Pin<Box<dyn Future<Output = (Self, Context<Self>)>>> {
           Box::pin(async move {
               context.log(&"You can log events!".to_string());

               let mut random = context.random().await;

               let roll = random.next_u64() % 6;

               // ...
           })
       }

       // ...
   }
   ```

6. Define the `serialize` and `deserialize` methods.

   Your game state possibly might not be serializable, e.g. if it embeds closures.
   However, you should always return a serialization whenever possible to optimize proof lengths.

   ```rust
   use serde::{Deserialize, Serialize};
   use serde_cbor::{from_slice, to_vec};

   // ...

   #[derive(Deserialize, Serialize, Clone)]
   struct MyGameState {
       // ...
   }

   impl State for MyGameState {
       // ...

       fn deserialize(data: &[u8]) -> Result<Self, String> {
           from_slice(data).map_err(|error| error.to_string())
       }

       fn serialize(&self) -> Option<Vec<u8>> {
           to_vec(self).ok()
       }

       // ...
   }
   ```

7. Implement `is_serializable` if possible.

   This step is optional, but strongly recommended whenever possible for performance reasons since the default implementation calls `serialize` in order to determine a state's serializability, which may be expensive.
   Your implementation of this method must agree with your implementation of the `serialize` method, i.e. if you return `true`, `serialize` must return a serialization.

   ```rust
   impl State for MyGameState {
       // ...

       fn is_serializable(&self) -> bool {
           true // MyGameState is always serializable
       }

       // ...
   }
   ```

8. Optionally override the default implementation of `certificate`.

   You can customize the default subkey signing message for your game here.

   ```rust
   use arcadeum::crypto::eip55;

   // ...

   impl State for MyGameState {
       // ...

       fn certificate(address: &crate::crypto::Address) -> String {
           format!("Sign to play my game!\n\n{}\n", eip55(address))
       }

       // ...
   }
   ```

9. Finally, optionally generate JavaScript bindings with the `arcadeum::bind!` macro.

   ```rust
   use arcadeum::bind;

   bind!(MyGameState);

   // ...
   ```
