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
export LIBCLANG_PATH=`echo /nix/store/*-clang-*-lib/lib` # doesn't work if you have more than one
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

#### view results

http://localhost:3000 → chain state → results → wins/draws/losses → player account → +
