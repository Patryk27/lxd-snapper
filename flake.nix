{
  description = "lxd-snapper: LXD snapshots, automated";

  inputs = {
    naersk = {
      url = "github:nmattia/naersk";
    };

    nixpkgs = {
      url = "github:nixos/nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
  };

  outputs = { self, naersk, nixpkgs, rust-overlay }:
    let
      build = { system, target, RUSTFLAGS }:
        let
          pkgs = import nixpkgs {
            inherit system;

            overlays = [
              rust-overlay.overlay
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
            ln -s "${./libs}" $out/libs
            ln -s "${./src}" $out/src
          '';

        in
        naersk'.buildPackage {
          inherit src RUSTFLAGS;

          doCheck = true;
          cargoTestOptions = args: args ++ [ "--workspace" ];
          CARGO_BUILD_TARGET = target;
        };

      check = { system }:
        import ./tests {
          inherit nixpkgs;

          lxd-snapper = self.defaultPackage."${system}";
        };

    in
    {
      defaultPackage = {
        i686-linux = build {
          system = "i686-linux";
          target = "i686-unknown-linux-musl";
          RUSTFLAGS = "";
        };

        x86_64-linux = build {
          system = "x86_64-linux";
          target = "x86_64-unknown-linux-musl";
          RUSTFLAGS = "-C relocation-model=dynamic-no-pic";
        };
      };

      checks = {
        i686-linux = check {
          system = "i686-linux";
        };

        x86_64-linux = check {
          system = "x86_64-linux";
        };
      };
    };
}
