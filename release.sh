#!/usr/bin/env bash

set -e

function build {
    let target="$1"
    let name="$2"

    echo "Building ${target}"
    nix build ".#defaultPackage.${target}"
    cp ./result/bin/lxd-snapper "lxd-snapper-${name}"
    rm result

    echo "Signing ${target}"
    gpg --sign "lxd-snapper-${name}"
}

build "i686-linux" "linux32"
build "x86_64-linux" "linux64"
