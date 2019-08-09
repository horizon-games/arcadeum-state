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
$ cd state
$ cargo test -- --nocapture
```

## Licence

GNU LGPL v3.0
