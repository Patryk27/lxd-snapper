{
  description = "lxd-snapper: LXD snapshots, automated";

  inputs = {
    gitignore = {
      url = "github:hercules-ci/gitignore";
      flake = false;
    };

    naersk = {
      url = "github:nmattia/naersk";

      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
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
            sha256 = "sha256-KCh2UBGtdlBJ/4UOqZlxUtcyefv7MH1neoVNV4z0nWs=";
          }).rust.override {
            targets = [ target ];
          };

          gitignoreSource = (pkgs.callPackage gitignore { }).gitignoreSource;

          buildPackage = (pkgs.callPackage naersk {
            rustc = rust;
          }).buildPackage;

        in
        buildPackage {
          inherit RUSTFLAGS;

          src = gitignoreSource ./.;
          doCheck = true;
          cargoTestOptions = args: args ++ [ "--all" ];
          CARGO_BUILD_TARGET = target;
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
    };
}
