{
  description = "lxd-snapper: LXD snapshots, automated";

  inputs = {
    gitignore = {
      url = "github:hercules-ci/gitignore";
      flake = false;
    };

    naersk = {
      url = "github:nmattia/naersk";
    };

    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-20.09";
    };

    nixpkgs-mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs = { self, gitignore, naersk, nixpkgs, nixpkgs-mozilla }:
    let
      build = { system, target, RUSTFLAGS }:
        let
          pkgs = (import nixpkgs) {
            inherit system;

            overlays = [
              (import "${nixpkgs-mozilla}")
            ];
          };

          rust = (pkgs.rustChannelOf {
            rustToolchain = ./rust-toolchain;
            sha256 = "GpVvKLc8e4l5URj7YsJfgm2OwsNw35zhpGD/9Jzdt2o=";
          }).rust.override {
            targets = [ target ];
          };

          gitignoreSource = (pkgs.callPackage gitignore { }).gitignoreSource;

          buildPackage = (pkgs.callPackage naersk {
            cargo = rust;
            rustc = rust;
          }).buildPackage;

        in buildPackage {
          inherit RUSTFLAGS;

          src = gitignoreSource ./.;
          doCheck = true;
          cargoTestOptions = args: args ++ [ "--all" ];
          CARGO_BUILD_TARGET = target;
        };

    in {
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
    };
}
