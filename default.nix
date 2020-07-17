let
  pkgs = import <nixpkgs> {
    overlays = [
      (import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz))
    ];
  };

  rust-pkg = (pkgs.rustChannelOf {
    rustToolchain = ./rust-toolchain;
  }).rust;

  gitignore =
    pkgs.callPackage
      (
        import (
          builtins.fetchGit {
            url = "https://github.com/hercules-ci/gitignore";
            rev = "647d0821b590ee96056f4593640534542d8700e5";
          }
        )
      ) { };

  naersk =
    pkgs.callPackage
      (
        import (
          builtins.fetchGit {
            url = "https://github.com/nmattia/naersk";
            rev = "d5a23213d561893cebdf0d251502430334673036";
          }
        )
      ) {
      cargo = rust-pkg;
      rustc = rust-pkg;
    };

in
naersk.buildPackage {
  src = gitignore.gitignoreSource ./.;
  doCheck = true;
  cargoTestOptions = args: args ++ [ "--all" ];
}
