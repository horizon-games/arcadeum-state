# arcadeum-chain

## bootstrap

install pre-commit hook

```
ln -s ../../.githooks/pre-commit .git/hooks
```

install rust nightly

```
./init.sh
```

## build

```
./build.sh
```

nixos-specific:

```
nix-shell -p openssl pkgconfig clang
export LIBCLANG_PATH=<location of libclang.so>
./build.sh
```

## run

### prover

```
yarn node examples/ttt/test/src/index.js
yarn node examples/coin/test/src/index.js
```

### blockchain

```
target/release/arcadeum-chain purge-chain --dev # optional
target/release/arcadeum-chain --dev
```

### blockchain viewer

```
git clone https://github.com/polkadot-js/apps.git
cd apps
yarn
yarn start
```

http://localhost:3000 → settings → local node (127.0.0.1:9944)

#### submit proof

http://localhost:3000 → extrinsics → alice → arcadeum → prove(proof) → proof → submit transaction

#### view records

http://localhost:3000 → chain state → records → wins/draws/losses → player account → +
