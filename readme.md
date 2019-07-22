# lxd-snapper

LXD + ZFS is a particularly beautiful combo, which allows one to
snapshot tons of running containers at once.

One problem emerges though: how do you manage them pesky snapshots when
there are hundreds of them? How do you get rid of all the old ones? And,
most importantly, how do you do it **reliably**?

That is the very question I have asked myself and this application is my
response. 

# Purpose

In a nutshell: creating and pruning LXD snapshots according to custom
policies.

# Users

- Me! (both on my private PC and server)

# Requirements

- LXD 2 / 3

# Getting started (using precompiled binaries)

```bash
# Download lxd-snapper:
$ wget https://github.com/Patryk27/lxd-snapper/releases/download/0.1/lxd-snapper-linux64 -O lxd-snapper

# Prepare default configuration:
$ cp docs/examples/keep-last-one.yml config.yml

# Make sure everything's fine:
$ ./lxd-snapper validate

# Create some them fancy snapshots:
$ ./lxd-snapper backup

# If you run above command a few times in a row, it will create a set of
# redundant snapshots that you can then delete with:
$ ./lxd-snapper prune

# Voilà!
```

# Getting started (compiling manually)

```bash
# Download and compile lxd-snapper:
$ git clone https://github.com/Patryk27/lxd-snapper
    && cd lxd-snapper
    && cargo build --release
    && cp target/release/lxd-snapper .
    
# Prepare default configuration:
$ cp docs/examples/keep-last-one.yml config.yml

# Make sure everything's fine:
$ ./lxd-snapper validate

# Create some them fancy snapshots:
$ ./lxd-snapper backup

# If you run above command a few times in a row, it will create a set of
# redundant snapshots that you can then delete with:
$ ./lxd-snapper prune

# Voilà!
```

# Setting up with cron

After you get used to `lxd-snapper` and prepare your favourite setup,
you would most likely want to set it up with `crontab` like so:

```
5 * * * * /root/lxd-snapper/lxd-snapper -c /root/lxd-snapper/config.yaml backup-prune
```

# Commands

## Backing-up containers / Creating snapshots

```bash
$ ./lxd-snapper backup
```

This command creates a stateless snapshot for each container that
matches policy specified in the `config.yml` file.

Each snapshot will be named in the `auto-$DATE-$TIME` fashion (e.g.
`auto-20190101-123000`).

You can execute `./lxd-snapper --dry-run backup` to perform a simulation
that will show what _would_ happen without actually creating any
snapshots.

## Pruning containers / Deleting snapshots

```bash
$ ./lxd-snapper prune
```

This command deletes all the **old**, automatically created (`auto-`)
snapshots from each container that matches policy specified in the
`config.yml` file, adhering to the `keep-` rules.

Only snapshots with names matching the `auto-$DATE-$TIME` format will be
pruned - all the other ones will be left untouched.

You can execute `./lxd-snapper --dry-run prune` to perform a simulation
that will show what _would_ happen without actually removing any
snapshots.

## Backing-up and then pruning at once

```bash
$ ./lxd-snapper backup-prune
```

This is a helper-command - it performs backing-up and then pruning (only
if backing-up was successful).

You can execute `./lxd-snapper --dry-run backup-prune` to perform a 
simulation that will show what _would_ happen without actually removing
any snapshots.

## Deleting ALL snapshots

```bash
$ ./lxd-snapper nuke
```

This command deletes **all** the snapshots from each container that
matches policy specified in the `config.yml` file.

This command is **destructive** - it completely ignores all the `keep-`
options and recklessly deletes all snapshots it can find (even the
non-auto-ones!). It does not touch the containers themselves though,
they will survive.

You can execute `./lxd-snapper --dry-run nuke` to perform a simulation
that will show what _would_ happen without actually removing any
snapshots.

## Checking configuration

```bash
$ ./lxd-snapper validate
```

This command will validate syntax of the configuration file and check
whether we can connect to the LXD daemon.

# Configuration file's syntax

All parameters are **optional** - you would most likely want to create
at least one `keep-` rule for pruning to work correctly though.

```yaml
# lxd-snapper tries to automatically guess where your LXD client (lxc) has been
# installed to. When it fails, you may help it by specifying its path manually.
#
# Example:
#   lxc-path: '/usr/bin/lxc'
lxc-path: String

policies:
  name-of-your-policy:
    # When specified, only containers with specified names will be backed up
    # and pruned.
    #
    # Example:
    #   allowed-containers: ['dancing-kangaroo', 'linus-torvalds']
    allowed-containers: String[]

    # When specified, only containers with specified statuses will be backed up
    # and pruned.
    #
    # Possible values:
    #  https://linuxcontainers.org/lxc/manpages/man7/lxc.7.html
    #  (the "Container life cycle" paragraph)
    #
    # Example:
    #   allowed-statuses: ['Running', 'Stopped']
    allowed-statuses: String[]

    # All of the "keep-" options determine pruning policy - they are basically
    # a rip off from Borg, so I will just drop you link to its documentation:
    #   https://borgbackup.readthedocs.io/en/stable/usage/prune.html
    keep-hourly: Integer
    keep-daily: Integer
    keep-weekly: Integer
    keep-monthly: Integer
    keep-yearly: Integer
    keep-last: Integer

    # Determines maximum number of snapshots (per container) there can be, but
    # does not create any snapshot of its own (contrary to the 'keep-last'
    # option).
    #
    # For example: if you set 'keep-yearly' to '100' and this option to '10', a
    # maximum number of '10' snapshots will be allowed to be created. All the
    # other ones will be pruned (starting with the oldest ones).
    #
    # Example:
    #   keep-limit: 64
    keep-limit: Integer
```

# Tips

## Running in a simulated, non-destructive mode

```bash
$ ./lxd-snapper --dry-run <command>
```

## Disabling colors

```bash
$ NO_COLOR=0 ./lxd-snapper
```

# Future plans

- Implement timeout-ing for the `LxdProcessClient` - currently if LXC
  gets stuck, so will the entire application.
  
- Implement integration tests (via Docker / Vagrant maybe?).

- Probably some small refactoring - right now a few parts of the
  application seem to be quite rough.
 
# Disclaimer

Snapshots are **not** a replacement for proper backup solution - to get
the best of both words, use snapshots and backups together, wisely.
 
# License

Copyright (c) 2019, Patryk Wychowaniec <wychowaniec.patryk@gmail.com>.    
Licensed under the MIT license.
