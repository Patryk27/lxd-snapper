{
  description = "lxd-snapper: LXD snapshots, automated";

  inputs = {
    naersk = {
      url = "github:nix-community/naersk";
    };

    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };

    # nixpkgs containing LXD 4, used for testing purposes
    nixpkgs--lxd-4 = {
      url = "github:nixos/nixpkgs/d1c3fea7ecbed758168787fe4e4a3157e52bc808";
    };

    # nixpkgs containing LXD 5, used for testing purposes
    nixpkgs--lxd-5 = {
      url = "github:nixos/nixpkgs/ee01de29d2f58d56b1be4ae24c24bd91c5380cea";
    };

    # nixpkgs containing LXD 6, used for testing purposes
    nixpkgs--lxd-6 = {
      url = "github:nixos/nixpkgs/4802ed07225c42ec290c86800ccf668807763567";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
  };

  outputs =
    { self
    , naersk
    , nixpkgs
    , nixpkgs--lxd-4
    , nixpkgs--lxd-5
    , nixpkgs--lxd-6
    , rust-overlay
    }:
    let
      mkPackage = { system, target, RUSTFLAGS }:
        let
          pkgs = import nixpkgs {
            inherit system;

            overlays = [
              rust-overlay.overlays.default
            ];
          };

          rust = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain).override {
            targets = [ target ];
          };

          naersk' = pkgs.callPackage naersk {
            cargo = rust;
            rustc = rust;
          };

          # Generates a new derivation without the ./tests directory; allows to
          # save a lot of time on incremental `nix flake check`-s, as otherwise
          # any change to any end-to-end test would force lxd-snapper to be
          # rebuilt from scratch.
          src = pkgs.runCommand "src" { } ''
            mkdir $out
            ln -s "${./Cargo.lock}" $out/Cargo.lock
            ln -s "${./Cargo.toml}" $out/Cargo.toml
            ln -s "${./docs}" $out/docs
            ln -s "${./src}" $out/src
          '';

        in
        naersk'.buildPackage {
          inherit src RUSTFLAGS;

          doCheck = true;
          CARGO_BUILD_TARGET = target;
        };

      mkCheck = { system }:
        import ./tests.nix {
          inherit
            nixpkgs
            nixpkgs--lxd-4
            nixpkgs--lxd-5
            nixpkgs--lxd-6;

          lxd-snapper = self.packages."${system}".default;
        };

    in
    {
      checks = {
        x86_64-linux = mkCheck {
          system = "x86_64-linux";
        };
      };

      packages = {
        x86_64-linux = {
          default = mkPackage {
            system = "x86_64-linux";
            target = "x86_64-unknown-linux-musl";
            RUSTFLAGS = "-C relocation-model=dynamic-no-pic";
          };
        };
      };
    };
}
