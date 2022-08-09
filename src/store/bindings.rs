/*
 * Copyright 2019 Horizon Blockchain Games Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! WebAssembly-specific utilities

use core::{convert::TryInto, num::NonZeroU32};

/// Generates WebAssembly bindings for a [super::State].
#[macro_export]
macro_rules! bind {
    ($type:ty) => {
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub struct WasmMatch {
            store: $crate::store::Store<$type>,
            send: js_sys::Function,
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl WasmMatch {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new(
                player: Option<$crate::Player>,
                root: &[u8],
                secret: wasm_bindgen::JsValue,
                p2p: bool,
                ready: js_sys::Function,
                sign: js_sys::Function,
                send: js_sys::Function,
                log: js_sys::Function,
                random: js_sys::Function,
            ) -> Result<WasmMatch, wasm_bindgen::JsValue> {
                Ok(Self {
                    store: {
                        $crate::store::Store::new(
                            player,
                            root,
                            match player {
                                None => $crate::utils::from_js(secret)?,
                                Some(0) => [Some($crate::utils::from_js(secret)?), None],
                                Some(1) => [None, Some($crate::utils::from_js(secret)?)],
                                _ => return Err("player.is_some() && player.unwrap() >= 2".into()),
                            },
                            p2p,
                            move |state, secrets| {
                                if let Ok(state) = $crate::utils::to_js(state) {
                                    match secrets {
                                        [Some(secret1), Some(secret2)] => {
                                            drop(
                                                ready.call3(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                    &$crate::utils::to_js(secret1)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                    &$crate::utils::to_js(secret2)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                ),
                                            );
                                        }
                                        [Some(secret), None] | [None, Some(secret)] => {
                                            drop(
                                                ready.call2(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                    &$crate::utils::to_js(secret)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                ),
                                            );
                                        }
                                        [None, None] => {
                                            drop(
                                                ready.call1(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                ),
                                            );
                                        }
                                    }
                                }
                            },
                            move |message| {
                                std::convert::TryInto::try_into($crate::utils::from_js::<Vec<_>>(
                                    sign.call1(
                                        &wasm_bindgen::JsValue::UNDEFINED,
                                        &$crate::utils::to_js(message)?,
                                    )
                                    .map_err(|error| format!("{:?}", error))?,
                                )?)
                                .map_err(|error| format!("{:?}", error))
                            },
                            {
                                let send = send.clone();

                                move |diff| {
                                    if let Ok(value) = &$crate::utils::to_js(&diff.serialize()) {
                                        drop(send.call1(&wasm_bindgen::JsValue::UNDEFINED, value));
                                    }
                                }
                            },
                            move |target, event| {
                                if let (Ok(target), Ok(event)) =
                                    ($crate::utils::to_js(&target), $crate::utils::to_js(&event))
                                {
                                    drop(log.call2(
                                        &wasm_bindgen::JsValue::UNDEFINED,
                                        &target,
                                        &event,
                                    ));
                                }
                            },
                            $crate::store::bindings::JsRng(random),
                        )?
                    },
                    send,
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn deserialize(
                data: &[u8],
                p2p: bool,
                ready: js_sys::Function,
                sign: js_sys::Function,
                send: js_sys::Function,
                log: js_sys::Function,
                random: js_sys::Function,
            ) -> Result<WasmMatch, wasm_bindgen::JsValue> {
                Ok(Self {
                    store: {
                        $crate::store::Store::deserialize(
                            data,
                            p2p,
                            move |state, secrets| {
                                if let Ok(state) = $crate::utils::to_js(state) {
                                    match secrets {
                                        [Some(secret1), Some(secret2)] => {
                                            drop(
                                                ready.call3(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                    &$crate::utils::to_js(secret1)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                    &$crate::utils::to_js(secret2)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                ),
                                            );
                                        }
                                        [Some(secret), None] | [None, Some(secret)] => {
                                            drop(
                                                ready.call2(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                    &$crate::utils::to_js(secret)
                                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                                ),
                                            );
                                        }
                                        [None, None] => {
                                            drop(
                                                ready.call1(
                                                    &wasm_bindgen::JsValue::UNDEFINED,
                                                    &state,
                                                ),
                                            );
                                        }
                                    }
                                }
                            },
                            move |message| {
                                std::convert::TryInto::try_into($crate::utils::from_js::<Vec<_>>(
                                    sign.call1(
                                        &wasm_bindgen::JsValue::UNDEFINED,
                                        &$crate::utils::to_js(message)?,
                                    )
                                    .map_err(|error| format!("{:?}", error))?,
                                )?)
                                .map_err(|error| format!("{:?}", error))
                            },
                            {
                                let send = send.clone();

                                move |diff| {
                                    if let Ok(value) = &$crate::utils::to_js(&diff.serialize()) {
                                        drop(send.call1(&wasm_bindgen::JsValue::UNDEFINED, value));
                                    }
                                }
                            },
                            move |target, event| {
                                if let (Ok(target), Ok(event)) =
                                    ($crate::utils::to_js(&target), $crate::utils::to_js(&event))
                                {
                                    drop(log.call2(
                                        &wasm_bindgen::JsValue::UNDEFINED,
                                        &target,
                                        &event,
                                    ));
                                }
                            },
                            $crate::store::bindings::JsRng(random),
                        )?
                    },
                    send,
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn serialize(&self, secret_knowledge: u8) -> Vec<u8> {
                self.store.serialize(match secret_knowledge {
                    1 => $crate::store::SecretKnowledge::Some(0),
                    2 => $crate::store::SecretKnowledge::Some(1),
                    3 => $crate::store::SecretKnowledge::Both,
                    _ => $crate::store::SecretKnowledge::None,
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn player(&self) -> Option<$crate::Player> {
                self.store.player()
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn id(&self) -> Vec<u8> {
                $crate::ID::serialize(self.store.state().id())
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn addresses(&self) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    &self
                        .store
                        .state()
                        .players()
                        .iter()
                        .map($crate::crypto::Addressable::eip55)
                        .collect::<Vec<_>>(),
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn hash(&self) -> String {
                $crate::utils::hex(self.store.hash())
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = hasState)]
            pub fn has_state(&self) -> bool {
                self.store.state().state().state().is_some()
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn state(&self) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    self.store
                        .state()
                        .state()
                        .state()
                        .ok_or("self.store.state().state().state().is_none()")?,
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = hasSecret)]
            pub fn has_secret(&self, player: $crate::Player) -> bool {
                self.store.state().state().secret(player).is_some()
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn secret(
                &self,
                player: $crate::Player,
            ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    &**self
                        .store
                        .state()
                        .state()
                        .secret(player)
                        .ok_or("self.store.state().state().secret(player).is_none()")?,
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter, js_name = pendingPlayer)]
            pub fn pending_player(&self) -> Result<Option<$crate::Player>, wasm_bindgen::JsValue> {
                Ok(self.store.pending_player()?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = getAddressPlayer)]
            pub fn address_player(
                &self,
                address: &str,
            ) -> Result<Option<$crate::Player>, wasm_bindgen::JsValue> {
                Ok(self.store.state().player(
                    std::convert::TryInto::<_>::try_into($crate::utils::unhex(address)?.as_slice())
                        .map_err(|error| format!("{}", error))?,
                    self.store.owner(),
                ))
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn simulate(
                &self,
                player: Option<$crate::Player>,
                action: wasm_bindgen::JsValue,
                using_secrets: wasm_bindgen::JsValue,
            ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    &self.store.state().state().simulate(
                        player,
                        &$crate::utils::from_js(action)?,
                        $crate::utils::from_js(using_secrets)?,
                    )?,
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn flush(&mut self) -> Result<(), wasm_bindgen::JsValue> {
                Ok(self.store.flush()?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn dispatch(
                &mut self,
                action: wasm_bindgen::JsValue,
            ) -> Result<(), wasm_bindgen::JsValue> {
                let diff = self.store.diff(vec![$crate::ProofAction {
                    player: self.store.player(),
                    action: $crate::PlayerAction::Play($crate::store::StoreAction::new(
                        $crate::utils::from_js(action)?,
                    )),
                }])?;

                self.send.call1(
                    &wasm_bindgen::JsValue::UNDEFINED,
                    &$crate::utils::to_js(&diff.serialize())?,
                )?;

                Ok(self.store.apply(&diff)?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = dispatchCertify)]
            pub fn dispatch_certify(
                &mut self,
                address: &str,
                signature: &str,
            ) -> Result<(), wasm_bindgen::JsValue> {
                let address =
                    std::convert::TryInto::<_>::try_into($crate::utils::unhex(address)?.as_slice())
                        .map_err(|error| format!("{}", error))?;

                if self
                    .store
                    .state()
                    .player(&address, self.store.owner())
                    .is_some()
                {
                    return Ok(());
                }

                let signature: $crate::crypto::Signature =
                    std::convert::TryInto::try_into($crate::utils::unhex(signature)?)
                        .map_err(|error| format!("{:?}", error))?;

                let player = self.store.state().player(&$crate::crypto::recover(<$crate::store::StoreState<$type> as $crate::State>::challenge(&address).as_bytes(), &signature)?, self.store.owner()).ok_or("self.store.state().player(&$crate::crypto::recover(<$crate::store::StoreState<$type> as $crate::State>::challenge(&address).as_bytes(), &signature)?, self.store.owner()).is_none()")?;

                let diff = self.store.diff(vec![$crate::ProofAction {
                    player: Some(player),
                    action: $crate::PlayerAction::Certify { address, signature },
                }])?;

                self.send.call1(
                    &wasm_bindgen::JsValue::UNDEFINED,
                    &$crate::utils::to_js(&diff.serialize())?,
                )?;

                Ok(self.store.apply(&diff)?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = dispatchApprove)]
            pub fn dispatch_approve(
                &mut self,
                player: &str,
                subkey: &str,
                signature: &str,
            ) -> Result<(), wasm_bindgen::JsValue> {
                let subkey =
                    std::convert::TryInto::<_>::try_into($crate::utils::unhex(subkey)?.as_slice())
                        .map_err(|error| format!("{}", error))?;

                if self
                    .store
                    .state()
                    .player(&subkey, self.store.owner())
                    .is_some()
                {
                    return Ok(());
                }

                let player =
                    std::convert::TryInto::<_>::try_into($crate::utils::unhex(player)?.as_slice())
                        .map_err(|error| format!("{}", error))?;

                let signature: $crate::crypto::Signature =
                    std::convert::TryInto::try_into($crate::utils::unhex(signature)?)
                        .map_err(|error| format!("{:?}", error))?;

                let diff = self.store.diff(vec![$crate::ProofAction {
                    player: None,
                    action: $crate::PlayerAction::Approve {
                        player,
                        subkey,
                        signature,
                    },
                }])?;

                self.send.call1(
                    &wasm_bindgen::JsValue::UNDEFINED,
                    &$crate::utils::to_js(&diff.serialize())?,
                )?;

                Ok(self.store.apply(&diff)?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = dispatchTimeout)]
            pub fn dispatch_timeout(&mut self) -> Result<(), wasm_bindgen::JsValue> {
                self.store
                    .dispatch_timeout()
                    .map_err(wasm_bindgen::JsValue::from)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn apply(&mut self, diff: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
                Ok(self.store.apply(&$crate::Diff::deserialize(diff)?)?)
            }
            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn raw_apply(&mut self, diff: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
                Ok(self.store.raw_apply(&$crate::Diff::deserialize(diff)?)?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = getApproval)]
            pub fn approval(player: &str, subkey: &str) -> Result<String, wasm_bindgen::JsValue> {
                Ok(
                    <$crate::store::StoreState<$type> as $crate::State>::approval(
                        std::convert::TryInto::<_>::try_into(
                            $crate::utils::unhex(player)?.as_slice(),
                        )
                        .map_err(|error| format!("{}", error))?,
                        std::convert::TryInto::<_>::try_into(
                            $crate::utils::unhex(subkey)?.as_slice(),
                        )
                        .map_err(|error| format!("{}", error))?,
                    ),
                )
            }
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        pub struct WasmState {
            state: $crate::store::StoreState<$type>,
            random: $crate::store::bindings::JsRng,
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl WasmState {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new(
                state: wasm_bindgen::JsValue,
                secrets: wasm_bindgen::JsValue,
                log: js_sys::Function,
                random: js_sys::Function,
            ) -> Result<WasmState, wasm_bindgen::JsValue> {
                let [secret1, secret2]: [Option<_>; 2] = $crate::utils::from_js(secrets)?;

                let secrets = [
                    secret1.map(|(secret, seed)| (secret, rand::SeedableRng::from_seed(seed))),
                    secret2.map(|(secret, seed)| (secret, rand::SeedableRng::from_seed(seed))),
                ];

                Ok(Self {
                    state: $crate::store::StoreState::new(
                        $crate::utils::from_js(state)?,
                        secrets,
                        move |target, event| {
                            drop(
                                log.call2(
                                    &wasm_bindgen::JsValue::UNDEFINED,
                                    &$crate::utils::to_js(&target)
                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                    &$crate::utils::to_js(&event)
                                        .unwrap_or(wasm_bindgen::JsValue::NULL),
                                ),
                            );
                        },
                    ),
                    random: $crate::store::bindings::JsRng(random),
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn deserialize(
                data: &[u8],
                log: js_sys::Function,
                random: js_sys::Function,
            ) -> Result<WasmState, wasm_bindgen::JsValue> {
                Ok(WasmState {
                    state: arcadeum::store::StoreState::deserialize(data, move |target, event| {
                        if let (Ok(target), Ok(event)) = (
                            arcadeum::utils::to_js(&target),
                            arcadeum::utils::to_js(&event),
                        ) {
                            drop(log.call2(&wasm_bindgen::JsValue::UNDEFINED, &target, &event))
                        }
                    })?,
                    random: arcadeum::store::bindings::JsRng(random),
                })
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn serialize(&self) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
                arcadeum::State::serialize(&self.state).ok_or(wasm_bindgen::JsValue::from(
                    "self.state.serialize().is_none()",
                ))
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = hasState)]
            pub fn has_state(&self) -> bool {
                self.state.state().is_some()
            }

            #[wasm_bindgen::prelude::wasm_bindgen(getter)]
            pub fn state(&self) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    self.state.state().ok_or("self.state.state().is_none()")?,
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen(js_name = hasSecret)]
            pub fn has_secret(&self, player: $crate::Player) -> bool {
                self.state.secret(player).is_some()
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn secret(
                &self,
                player: $crate::Player,
            ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(
                    &**self
                        .state
                        .secret(player)
                        .ok_or("self.state.secret(player).is_none()")?,
                )?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn simulate(
                &self,
                player: Option<$crate::Player>,
                action: wasm_bindgen::JsValue,
                using_secrets: wasm_bindgen::JsValue,
            ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
                Ok($crate::utils::to_js(&self.state.simulate(
                    player,
                    &$crate::utils::from_js(action)?,
                    $crate::utils::from_js(using_secrets)?,
                )?)?)
            }

            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn apply(
                &mut self,
                player: Option<$crate::Player>,
                action: wasm_bindgen::JsValue,
            ) -> Result<(), wasm_bindgen::JsValue> {
                Ok(self.state.apply_with_random(
                    player,
                    $crate::utils::from_js(action)?,
                    &mut self.random,
                )?)
            }
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getVersion)]
        pub fn version() -> String {
            $crate::utils::hex(<$crate::store::StoreState<$type> as $crate::State>::version())
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getChallenge)]
        pub fn challenge(address: &str) -> Result<String, wasm_bindgen::JsValue> {
            Ok(
                <$crate::store::StoreState<$type> as $crate::State>::challenge(
                    std::convert::TryInto::<_>::try_into($crate::utils::unhex(address)?.as_slice())
                        .map_err(|error| format!("{}", error))?,
                ),
            )
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getApproval)]
        pub fn approval(player: &str, subkey: &str) -> Result<String, wasm_bindgen::JsValue> {
            Ok(
                <$crate::store::StoreState<$type> as $crate::State>::approval(
                    std::convert::TryInto::<_>::try_into($crate::utils::unhex(player)?.as_slice())
                        .map_err(|error| format!("{}", error))?,
                    std::convert::TryInto::<_>::try_into($crate::utils::unhex(subkey)?.as_slice())
                        .map_err(|error| format!("{}", error))?,
                ),
            )
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getRootProofVersion)]
        pub fn root_proof_version(root: &[u8]) -> Result<String, wasm_bindgen::JsValue> {
            Ok($crate::utils::hex(&$crate::RootProof::<
                $crate::store::StoreState<$type>,
            >::version(root)?))
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getRootProofID)]
        pub fn root_proof_id(root: &[u8]) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
            Ok($crate::ID::serialize(
                $crate::RootProof::<$crate::store::StoreState<$type>>::deserialize(root)?
                    .state()
                    .id(),
            ))
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getRootProofPlayer)]
        pub fn root_proof_player(
            root: &[u8],
            player: &str,
        ) -> Result<$crate::Player, wasm_bindgen::JsValue> {
            let player = $crate::utils::unhex(player)?;

            if player.len() != std::mem::size_of::<$crate::crypto::Address>() {
                return Err(
                    "player.len() != std::mem::size_of::<$crate::crypto::Address>()".into(),
                );
            }

            let root = $crate::RootProof::<$crate::store::StoreState<$type>>::deserialize(root)?;

            root.state()
                .player(
                    std::convert::TryInto::<_>::try_into(player.as_slice())
                        .map_err(|error| format!("{}", error))?,
                    root.author(),
                )
                .ok_or("root.state().player(player, owner).is_none()".into())
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getDiffProof)]
        pub fn diff_proof(diff: &[u8]) -> Result<String, wasm_bindgen::JsValue> {
            Ok($crate::utils::hex(
                $crate::Diff::<$crate::store::StoreState<$type>>::deserialize(diff)?.proof(),
            ))
        }

        #[wasm_bindgen::prelude::wasm_bindgen(js_name = getDiffDebugString)]
        pub fn diff_debug_string(diff: &[u8]) -> Result<String, wasm_bindgen::JsValue> {
            Ok(format!(
                "{:?}",
                $crate::Diff::<$crate::store::StoreState<$type>>::deserialize(diff)?
            ))
        }
    };
}

#[doc(hidden)]
/// Random number generator using an external JavaScript function for entropy
pub struct JsRng(pub js_sys::Function);

impl rand::RngCore for JsRng {
    fn next_u32(&mut self) -> u32 {
        rand_core::impls::next_u32_via_fill(self)
    }

    fn next_u64(&mut self) -> u64 {
        rand_core::impls::next_u64_via_fill(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.try_fill_bytes(dest).unwrap();
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        let length: u32 = dest
            .len()
            .try_into()
            .map_err(|_| NonZeroU32::new(rand::Error::CUSTOM_START).unwrap())?;

        let random: Vec<u8> = crate::utils::from_js(
            self.0
                .call1(&wasm_bindgen::JsValue::UNDEFINED, &length.into())
                .map_err(|_| NonZeroU32::new(rand::Error::CUSTOM_START + 1).unwrap())?,
        )
        .map_err(|_| NonZeroU32::new(rand::Error::CUSTOM_START + 2).unwrap())?;

        if random.len() != dest.len() {
            return Err(NonZeroU32::new(rand::Error::CUSTOM_START + 3)
                .unwrap()
                .into());
        }

        dest.copy_from_slice(&random);

        Ok(())
    }
}
