{ mozrev  ? "d46240e8755d91bc36c0c38621af72bf5c489e13"
, mozsha  ? "0icws1cbdscic8s8lx292chvh3fkkbjp571j89lmmha7vl2n71jg"
, sources ? import ./nix/sources.nix
, mozilla-overlay ? import (builtins.fetchTarball {
    url = "https://github.com/mozilla/nixpkgs-mozilla/archive/${mozrev}.tar.gz";
    sha256 = mozsha;
  })
, pkgs   ?
  import sources.nixpkgs {
    overlays = [mozilla-overlay];
  }
}:

with pkgs;

let
  rustChannels =
    pkgs.lib.mapAttrs
      (_: v: pkgs.rustChannelOf v)
      (import ./nix/rust-channels.nix {
        stableVersion = "1.38.0";
      });
  rustPlatform = makeRustPlatform {
    rustc = rustChannels.nightly.rust;
    cargo = rustChannels.nightly.rust;
  };
in

rustPlatform.buildRustPackage rec {
  pname = "advent-of-code-2019";
  version = "1.0.0";

  src = ./.;

  cargoSha256 = "0jacm96l1gw9nxwavqi1x4669cg6lzy9hr18zjpwlcyb3qkw9z7f";
  cargoSha256Version = 2;
  cargoBuildFlags = [];

  nativeBuildInputs = [];
  buildInputs = [ cargo rustfmt carnix ];

  preFixup = ''
  '';

  meta = with stdenv.lib; {
    description = "Fun with Advent of code in rust";
    homepage = https://github.com/jfroche/rust-advent-of-code-2019;
    license = with licenses; [ mit ];
    maintainers = [ maintainers.jfroche ];
    platforms = platforms.all;
  };
}
