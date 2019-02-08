# arcadeum-chain

## bootstrap

install https://rustup.rs/

```
rustup default nightly
./init.sh
```

## build

```
./build.sh
```

nixos-specific:

```
nix-shell -p openssl pkgconfig clang
# add libstdc++.so location to LD_LIBRARY_PATH
# add libclang.so location to LIBCLANG_PATH
./build.sh
```

## run

### blockchain

```
target/release/arcadeum-chain --dev
```

### blockchain explorer

https://polkadot.js.org/apps (chromium only, doesn't work on firefox)

settings: localhost:9944

chain state: view Alice records wins, losses, and draws

### wallet ui

in https://github.com/paritytech/substrate-ui, branch `substrate-node-template`:

```
yarn && yarn dev
```

go to http://localhost:8000:

add seed `0x416c696365202020202020202020202020202020202020202020202020202020`, name `Alice`

open js console:

send a win for Alice (X wins):

```
post({ sender: '5GoKvZWG5ZPYL1WUovuHW3zJBWBP5eT8CbqjdRY4Q6iMaDtZ', call: calls.arcadeum.prove(new Uint8Array([3, 0, 0, 0, 1, 0, 0, 3, 0, 0, 0, 2, 1, 0, 3, 0, 0, 0, 1, 0, 1, 3, 0, 0, 0, 2, 1, 1, 3, 0, 0, 0, 1, 0, 2])) }).tie(console.log)
```

send a loss for Alice (O wins):

```
post({ sender: '5GoKvZWG5ZPYL1WUovuHW3zJBWBP5eT8CbqjdRY4Q6iMaDtZ', call: calls.arcadeum.prove(new Uint8Array([3, 0, 0, 0, 1, 0, 0, 3, 0, 0, 0, 2, 1, 0, 3, 0, 0, 0, 1, 0, 1, 3, 0, 0, 0, 2, 1, 1, 3, 0, 0, 0, 1, 2, 0, 3, 0, 0, 0, 2, 1, 2])) }).tie(console.log)
```

send a draw for Alice:

```
post({ sender: '5GoKvZWG5ZPYL1WUovuHW3zJBWBP5eT8CbqjdRY4Q6iMaDtZ', call: calls.arcadeum.prove(new Uint8Array([3, 0, 0, 0, 1, 1, 1, 3, 0, 0, 0, 2, 0, 0, 3, 0, 0, 0, 1, 0, 2, 3, 0, 0, 0, 2, 2, 0, 3, 0, 0, 0, 1, 1, 0, 3, 0, 0, 0, 2, 1, 2, 3, 0, 0, 0, 1, 0, 1, 3, 0, 0, 0, 2, 2, 1, 3, 0, 0, 0, 1, 2, 2])) }).tie(console.log)
```

### bindings example

```
cd example
yarn node src/index.js
```
