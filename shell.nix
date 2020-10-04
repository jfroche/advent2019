with import <nixpkgs> { };

let
  sources = import ./nix/sources.nix;
  mozilla-overlay =
    import
  (
    builtins.fetchTarball
    https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz
  );
  pkgs = import sources.nixpkgs {
    overlays = [mozilla-overlay];
  };
  rustChannels =
    pkgs.lib.mapAttrs
      (_: v: pkgs.rustChannelOf v)
      (import ./nix/rust-channels.nix {
        stableVersion = "1.46.0";
      });
in pkgs.mkShell rec {
  name = "advent-of-code";

  buildInputs = [
    # dev tools
    #pkgs.cargo-edit
    pkgs.cargo-release
    pkgs.rustfmt
    rustChannels.nightly.rust
    pkgs.rustup
    pkgs.carnix
    pkgs.httpie
    gitAndTools.pre-commit
    pkgs.rustracer
    pkgs.graphviz
    pkgs.rls
    pkgs.niv
    pkgs.clippy
  ];
  RUST_SRC_PATH= "${rustChannels.stable.rust-src}/lib/rustlib/src/rust/src";
  HISTFILE = "${toString ./.}/.bash_history";
  RUST_BACKTRACE = 1;
  RUST_LOG="INFO";
  ADVENT_COOKIE_SESSION="<REDACTED>";
}
