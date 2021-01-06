# lxd-snapper

**LXD snapshots, automated.**

lxd-snapper is a tool that automates creating & removing LXD snapshots: just setup a snapshotting policy, add a systemd
timer (or cron, or whatever scheduling mechanism you use) and enjoy your containers.

tl;dr it's a fancy wrapper for `lxc snapshot` & `lxc delete`; kinda like LXD's built-in `snapshots.schedule`, just more
configurable.

# Requirements

- LXD 4+
- Linux (i386 or x86_64)

Plus, if you plan on compiling this application locally:

- Cargo & Rust 1.49+ (nightly)

# Getting started

## Downloading

You can either download pre-built binaries:

```bash
# i386:
$ wget https://github.com/Patryk27/lxd-snapper/releases/download/v1.0/lxd-snapper-linux32 -O lxd-snapper
$ chmod u+x lxd-snapper

# x86_64:
$ wget https://github.com/Patryk27/lxd-snapper/releases/download/v1.0/lxd-snapper-linux64 -O lxd-snapper
$ chmod u+x lxd-snapper
```

... or build it on your own:

```bash
# Using Cargo:
$ git clone https://github.com/Patryk27/lxd-snapper
    && cd lxd-snapper
    && cargo build --release
    && ./target/release/lxd-snapper

# Or using Nix (v2):
$ git clone https://github.com/Patryk27/lxd-snapper
    && cd lxd-snapper
    && nix-build
    && ./result/bin/lxd-snapper

# Or using Nix (v3):
$ git clone https://github.com/Patryk27/lxd-snapper
    && cd lxd-snapper
    && nix build
    && ./result/bin/lxd-snapper
```

## Configuring

lxd-snapper requires a single configuration file, written in YAML, that defines **policies**; each policy describes
which LXD _instances_ (or _projects_, or _statuses_) it matches, and defines retention strategies for snapshots of those
matching instances (so _containers_ or _virtual machines_).

Being practical, let's start with a minimal configuration:

```yaml
policies:
  my-first-policy:
    keep-last: 5
```

This configuration defines a single policy that applies to _all_ of the instances (inside _all_ of the projects, if you
use LXD projects) and basically tells lxd-snapper that for each instance you want to keep five of its _latest_ snapshots
around.

You can go ahead and try it locally by saving that configuration into `config.yaml` and running
`lxd-snapper -c config.yaml --dry-run backup` - thanks to the `--dry-run` switch, no changes will be actually applied,
you'll just see what _would_ happen.

----

Policies can be applied to a **subset of all the instances**:

```yaml
policies:
  databases:
    included-instances: ['mysql']
    keep-last: 8

  files:
    included-instances: ['nginx', 'php']
    keep-last: 1
```

This configuration defines two policies:

- the first one applies to instance named `mysql` and tells lxd-snapper that it should keep its eight latest snapshots,
- the second one applies to instances named `nginx` and `php`, and tells lxd-snapper that you care only about their
  very latest snapshots.

_(there's also `excluded-instances`, which works symmetrically.)_

----

Policies can be applied to a **subset of all the projects**:

```yaml
policies:
  important-clients:
    included-projects: ['client-a', 'client-b']
    keep-last: 32

  unimportant-clients:
    included-projects: ['client-c']
    keep-last: 2
```

This configuration, too, defines two policies:

- the first one applies to all of the instances inside the `client-a` and `client-b` projects (and tells lxd-snapper to
  keep the latest thirty two snapshots per instance),
- the second one applies to all of the instances inside the `client-c` project.

_(there's also `excluded-projects`, which works symmetrically.)_

----

Apart from `keep-last`, there are many other `keep-` strategies you can use:

```yaml
policies:
  all-instances:
    keep-daily: 15
    keep-weekly: 10
    keep-monthly: 5
    keep-yearly: 4
```

If you're familiar with Borg (especially the [`borg prune`](https://borgbackup.readthedocs.io/en/stable/usage/prune.html)
command), those names shouldn't come as a surprise - lxd-snapper implements the same retention algorithm as Borg does.

On the other hand, if those names _do_ seem scary, don't worry - what they describe is that all instances will be kept
15 snapshots for the recent days, 10 snapshots for the recent weeks, 5 snapshots for the recent months, and 4 snapshots
for the recent years.

In total, this counts for 15 + 10 + 5 + 4 = **34** snapshots _per instance, with the newest snapshot being the one from
today, and the oldest - the one from four~five years.

Though this algorithm might be a bit hard to grasp, it's amazingly versatile and allows for lots of different
combinations; I encourage you to try going through the example configs inside `docs/example-configs` to see more
examples with some in-depth commentary about them.

----

As the last feature, policies are **cascading**:

```yaml
policies:
  important-clients:
    included-projects: ['client-a', 'client-b']
    keep-hourly: 6
    keep-daily: 5
    keep-monthly: 2
    keep-yearly: 1

  unimportant-clients:
    included-projects: ['client-c']
    keep-hourly: 6
    keep-daily: 2
    keep-monthly: 1

  databases:
    included-instances: ['mysql', 'mongodb']
    keep-hourly: 12
```

Let's say that we have a `mysql` instance inside the `client-c` project - it matches two of our policies:
`unimportant-clients` and `databases`. The way lxd-snapper evaluates strategy for that particular instance is that it
takes properties from both `unimportant-clients` and `databases`, and squashes them together, overwriting duplicated
fields.

In this case we'd end up with an equivalent of:

```yaml
keep-hourly: 12 # taken from `databases`
keep-daily: 2   # taken from `unimportant-clients`
keep-monthly: 1 # taken from `unimportant-clients`
```

Policies are always evaluated top-bottom, so that the policy that's "below" will overwrite properties of policy "above"
it, in case when a instance matches many of them.

This is quite a useful feature, because it allows you to extract common rules to the top of the configuration file; for
instance, on my server I'm running a configuration similar to:

```yaml
config:
  _all:
    keep-hourly: 6
    keep-daily: 5
    keep-monthly: 4
    keep-yearly: 1

  _non_essential:
    included-instances: ['gitlab-runner']
    keep-limit: 4

  grafana:
    included-instances: ['grafana', 'grafana-influx']
    keep-hourly: 12
```

Thanks to the cascading, I'm able to provide special rules for "non essential" instances like `gitlab-runner` (which I
just wouldn't like to be backed-up as much as other instances).

## Usage

Assuming you've prepared a proof-of-concept of your configuration and saved into `config.yaml`, you can do:

```
./lxd-snapper query instances
```

This command will list all the instances and show you which policies match them:

```
+---------+----------+---------------------------+
| Project | Instance | Policies                  |
+=========+==========+===========================+
| default | nginx    | NONE                      |
| default | mongodb  | databases + non-essential |
| default | mysql    | databases                 |
+---------+----------+---------------------------+
```

_(`+` means that many policies match given instance and cascading will apply.)_

If all policies are matched correctly (watch for typos!), you can do:

```shell
$ ./lxd-snapper --dry-run backup
```

This command will perform a "dry run" of the `backup` command, meaning that you'll see what _would_ happen without
actually applying any change to any of the instances:

```
note: --dry-run is active, no changes will be applied

Backing-up instances:

- default/mongodb
-> creating snapshot: auto-20200722-165621
-> [ OK ]

- default/mysql
-> creating snapshot: auto-20200722-165621
-> [ OK ]

Summary
- processed instances: 3
- created snapshots: 2
```

_(all commands support the `--dry-run` switch - you can use it any time to preview what would happen if you run a particular command.)_

If everything looks good, you can perform the actual backup:

```shell
$ ./lxd-snapper backup
```

This command will run `lxc snapshot` for each instance that matches at least one policy; continuing the example from
above, it would be similar to invoking `lxc snapshot mysql` and `lxc snapshot mongodb`. Containers that don't match any
policy (like the `nginx` instance above) won't be touched.

**`backup` never removes any snapshots** - in order to remove old snapshots, you have to run `prune`:

```shell
$ ./lxd-snapper --dry-run prune

note: --dry-run is active, no changes will be applied

Pruning instances:

- default/mongodb
-> keeping snapshot: auto-20200722-165621
-> [ OK ]

- default/mysql
-> keeping snapshot: auto-20200722-165621
-> [ OK ]

Summary
- processed instances: 3
- deleted snapshots: 0
- kept snapshots: 2
```

Once again, if everything seems fine, perform the actual pruning:

```shell
$ ./lxd-snapper prune
```

This command will iterate through all the instances and remove their stale snapshots (i.e. the ones that are over the
`keep-` limits) using the `lxc delete` command.

Only snapshots matching the `snapshot-name-prefix` setting will be accounted for - so, by default, only those snapshots
whose names begin with `auto-` are counted towards the limits and are candidates for removal; lxd-snapper won't
automatically remove other (e.g. hand-made) snapshots.

Usually, to keep snapshots on-time & tidy, you'll want run `backup` and `prune` together, like so:

```shell
$ ./lxd-snapper backup-and-prune
```

## Scheduling

lxd-snapper is a fire-and-forget application - it doesn't daemonize itself; to keep instances backed-up & pruned on
time, you'll most likely want to create a systemctl timer or a cron job for it:

```
5 * * * * /usr/bin/lxd-snapper -c /etc/lxd-snapper.yaml backup-and-prune
```

# Synopsis

```
lxd-snapper
LXD snapshots, automated

USAGE:
    lxd-snapper [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -d, --dry-run    Runs application in a simulated safe-mode without applying any changes to the instances
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>        Path to the configuration file [default: config.yaml]
    -l, --lxc-path <lxc-path>    Path to the `lxc` executable; usually inferred automatically from the `PATH`
                                 environmental variable

SUBCOMMANDS:
    backup              Creates a snapshot for each instance matching the policy
    backup-and-prune    Shorthand for `backup` followed by `prune`
    debug               Various debug-oriented commands
    help                Prints this message or the help of the given subcommand(s)
    prune               Removes stale snapshots for each instance matching the policy
    query               Various query-oriented commands
    validate            Validates policy's syntax
```

# Configuration file syntax

```yaml
snapshot-name-prefix: 'auto-'

policies:
  policy:
    # When set, this policy will apply only to these
    # specific projects.
    #
    # By default each policy applies to all projects.
    included-projects: ['project-a', 'project-b']

    # Symmetric to `included-projects`; when set, policy
    # will apply to all projects _except_ these specific
    # ones.
    excluded-projects: ['project-c', 'project-d']

    # When set, this policy will apply only to these
    # specific instances (containers & virtual machines).
    #
    # By default each policy applies to all instances.
    included-instances: ['container-a', 'virtual-machine-b']

    # Symmetric to `included-instances`; when set, policy
    # will apply to all instances _except_ these specific
    # ones.
    excluded-instances: ['container-c', 'virtual-machine-d']

    # When set, this policy will apply only to instances
    # with given status.
    #
    # Possible values: Aborting, Running, Starting,
    # Stopped, and Stopping.
    #
    # By default each policy applies to all statuses.
    included-statuses: ['Running']

    # Symmetric to `included-statuses`; when set, policy
    # will apply to all statuses _except_ these specific
    # ones.
    excluded-statuses: ['Stopped']

    # All the `keep-` properties below determine retention
    # policy. The mechanism is a rip off of Borg, so:
    # https://borgbackup.readthedocs.io/en/stable/usage/prune.html
    #
    # You'll find examples inside `docs/example-configs`
    # useful too.

    keep-hourly: 1
    keep-daily: 1
    keep-weekly: 1
    keep-monthly: 1
    keep-yearly: 1
    keep-last: 1
    keep-limit: 1
```

# Edge cases worth knowing about

- When an instance inside `included-instances` / `excluded-instances` or project inside `included-projects` /
  `excluded-projects` refers to an entity that does not exist (say, `included-instances: ['rick-roll']`), the missing
  instance or project is silently ignored; this is by design.

# Disclaimer

Snapshots are _not_ a replacement for backups - to keep your data safe, use snapshots and backups together, wisely.

# License

Copyright (c) 2019-2021, Patryk Wychowaniec <pwychowaniec@pm.me>.    
Licensed under the MIT license.
