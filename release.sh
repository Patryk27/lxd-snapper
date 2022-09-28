#!/usr/bin/env bash

set -e

function build {
    local target="${1}"
    local name="lxd-snapper-${2}"

    echo "Building ${target}"
    nix build ".#packages.${target}.default"
    cp ./result/bin/lxd-snapper "${name}"
    rm result

    echo "Signing ${target}"
    gpg --output "${name}.sig" --detach-sig "${name}"
}

build "i686-linux" "linux32"
build "x86_64-linux" "linux64"
