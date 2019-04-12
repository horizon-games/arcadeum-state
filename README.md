# arcadeum-chain

## bootstrap

install pre-commit hook

```
ln -s ../../.githooks/pre-commit .git/hooks
```

install rust nightly

```
substrate/init.sh
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

### blockchain

```
substrate/target/release/arcadeum-chain purge-chain --dev # optional
substrate/target/release/arcadeum-chain --dev
```

### prover

```
yarn node examples/coin/test/src/index.js
```

or:

```
yarn node examples/ttt/test/src/index.js # must edit game dependency in substrate/runtime/Cargo.toml and re-build
```

### blockchain viewer

```
git clone https://github.com/polkadot-js/apps.git
cd apps
yarn
yarn start
```

http://localhost:3000 → settings → local node (127.0.0.1:9944)

#### view results

http://localhost:3000 → chain state → results → wins/draws/losses → player account → +
