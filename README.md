# arcadeum

## Bootstrap

### Install pre-commit hook

```bash
$ ln -s ../../.githooks/pre-commit .git/hooks
```

### Install Rust nightly

#### NixOS

```
nixpkgs.overlays = [
  (import (builtins.fetchGit {
    url = "https://github.com/mozilla/nixpkgs-mozilla.git";
    ref = "master";
  }))
];
```

Add `pkgs.latest.rustChannels.nightly.rust` to packages.

`nix-shell -p` when building.

#### Other

See https://rustup.rs/.

```
$ rustup install nightly
$ rustup default nightly
```

## Build and test

### Quick test

```bash
$ cargo test -- --nocapture
```

### Thorough test

```bash
$ cargo test && cargo test --no-default-features && cargo test --features 'no-crypto' && cargo test --no-default-features --features 'no-crypto' && cargo test --features 'test-approvals' && cargo test --no-default-features --features 'test-approvals' && cargo test --features 'no-crypto, test-approvals' && cargo test --no-default-features --features 'no-crypto, test-approvals'
```

### Test no-std

```bash
$ cd std-tester && cargo run
```

## Documentation

```bash
$ cargo doc --release --no-deps && rm -rf docs/api && mv target/doc docs/api
```

## Licence

GNU LGPL v3.0
