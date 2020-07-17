# lxd-snapper

**LXD snapshots, automated.**

# Requirements

- LXD 2+

# Getting started

## Using precompiled binaries

```bash
# Download lxd-snapper
$ wget https://github.com/Patryk27/lxd-snapper/releases/download/0.2/lxd-snapper-linux64 -O lxd-snapper

# Prepare default configuration
$ cp docs/example-configs/basic-last.yaml config.yaml

# Make sure everything's fine
$ ./lxd-snapper validate

# Create some them fancy snapshots
$ ./lxd-snapper backup

# If you run above command a few times in a row, it will create a set of
# redundant snapshots that you can then delete with
$ ./lxd-snapper prune
```

## Compiling manually (without Nix)

```bash
# Download and compile lxd-snapper
$ git clone https://github.com/Patryk27/lxd-snapper
    && cd lxd-snapper
    && cargo build --release
    && cp target/release/lxd-snapper .

# Prepare default configuration
$ cp docs/example-configs/basic-last.yaml config.yaml

# Make sure everything's fine
$ ./lxd-snapper validate

# Create some them fancy snapshots
$ ./lxd-snapper backup

# If you run above command a few times in a row, it will create a set of
# redundant snapshots that you can then delete with
$ ./lxd-snapper prune

# Voilà!
```

## Compiling manually (with Nix)

```bash
# Download and compile lxd-snapper
$ git clone https://github.com/Patryk27/lxd-snapper
    && cd lxd-snapper
    && nix-build
    && cp result/bin/lxd-snapper .

# Prepare default configuration
$ cp docs/example-configs/basic-last.yaml config.yaml

# Make sure everything's fine
$ ./lxd-snapper validate

# Create some them fancy snapshots
$ ./lxd-snapper backup

# If you run above command a few times in a row, it will create a set of
# redundant snapshots that you can then delete with
$ ./lxd-snapper prune

# Voilà!
```

# Synopsis

TODO

# Configuration file's syntax

TODO

# Tips

## Running in a simulated, non-destructive mode

```bash
$ ./lxd-snapper --dry-run <command>
```

## Setting up with cron

After you get used to `lxd-snapper` and prepare your favourite setup,
you would most likely want to set it up with `crontab` like so:

```
5 * * * * /root/lxd-snapper/lxd-snapper -c /root/lxd-snapper/config.yaml backup-and-prune
```

# Disclaimer

Snapshots are **not** a replacement for proper backup solution - to get
the best of both words, use snapshots and backups together, wisely.
 
# License

Copyright (c) 2019-2020, Patryk Wychowaniec <wychowaniec.patryk@gmail.com>.
Licensed under the MIT license.