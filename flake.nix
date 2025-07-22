{
  description = "lxd-snapper: LXD/Incus snapshots, automated";

  inputs = {
    naersk = {
      url = "github:nix-community/naersk";
    };

    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
  };

  outputs =
    {
      self,
      naersk,
      nixpkgs,
      rust-overlay,
    }:
    let
      mkPackage =
        {
          system,
          target,
          RUSTFLAGS,
        }:
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

      mkCheck =
        { system }:
        import ./tests.nix {
          inherit
            nixpkgs
            ;

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
