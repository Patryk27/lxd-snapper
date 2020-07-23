let
  pkgs = import <nixpkgs> { };

in
pkgs.mkShell {
  buildInputs = with pkgs; [
    lld
    nixpkgs-fmt
    rustup
  ];
}
