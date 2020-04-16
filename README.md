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

```bash
$ cargo test --features tester --features bindings -- --nocapture
```

## Documentation

```bash
$ cargo doc --release --no-deps --features 'version bindings tester debug' && rm -rf docs/api && mv target/doc docs/api
```

## Licence

GNU LGPL v3.0
