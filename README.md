# lxd-snapper

**LXD snapshots, automated.**

lxd-snapper is a tool that automates creating & removing LXD snapshots: just
prepare a snapshotting policy, setup a cronjob, and enjoy your containers.

tl;dr it's a fancy wrapper for `lxc snapshot` & `lxc delete`; kinda like LXD's
built-in `snapshots.schedule`, just more powerful.

# Requirements

- LXD 4+
- Linux (i386 or x86_64)

Plus, if you plan on compiling lxd-snapper locally:

- Cargo & Rust 1.56.1+
- Nix (for running integration tests)

# Getting started

## Downloading

You can either download pre-built binaries:

``` console
# i686
$ wget https://github.com/Patryk27/lxd-snapper/releases/download/v1.2/lxd-snapper-linux32 -O lxd-snapper
$ chmod u+x lxd-snapper

# x86_64
$ wget https://github.com/Patryk27/lxd-snapper/releases/download/v1.2/lxd-snapper-linux64 -O lxd-snapper
$ chmod u+x lxd-snapper
```

... or build lxd-snapper on your own:

``` console
$ git clone https://github.com/Patryk27/lxd-snapper
$ cd lxd-snapper

# Using Cargo
$ cargo build --release
$ ./target/release/lxd-snapper

# (or) Using Nix v1
$ nix-build
$ ./result/bin/lxd-snapper

# (or) Using Nix v2 / v3
$ nix build
$ ./result/bin/lxd-snapper
```

## Configuring

Setting-up lxd-snapper is pretty easy: you just have to prepare a single
configuration file that will describe which LXD instances (so containers and/or
virtual machines) you want to get snapshotted, and for how long those snapshots
should be kept around.

We can start with the most basic configuration:

``` yaml
policies:
  my-first-policy:
    keep-last: 2
```

This beautiful YAML file defines a single policy called `my-first-policy` that
will snapshot all of your instances, keeping around the latest two snapshots per
each instance.

To make it less abstract, let's go ahead and create some containers:

``` console
$ lxc launch ubuntu: hello
$ lxc launch ubuntu: world
# (the container's OS doesn't matter - Ubuntu is just an example)

$ lxc ls
+-------+---------+------+------+-----------+-----------+
| NAME  |  STATE  | IPV4 | IPV6 |   TYPE    | SNAPSHOTS |
+-------+---------+------+------+-----------+-----------+
| hello | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+
| world | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+
```

Now, to snapshot those containers, first you'd have to store that configuration
from before into a file - say, `config.yaml` - and then run `lxd-snapper
backup`:

``` console
$ lxd-snapper --dry-run -c config.yaml backup
note: --dry-run is active, no changes will be applied

Backing-up instances:

- default/hello
-> creating snapshot: auto-20180101-120000
-> [ OK ]

- default/world
-> creating snapshot: auto-20180101-120000
-> [ OK ]

Summary
- processed instances: 2
- created snapshots: 2
```

As you can see, there's a detailed output of everything that's happened - or
rather of everything that _would_ happen: we used a magic switch called
`--dry-run` which tells lxd-snapper that you only want to **preview** the
changes without actually creating or removing any snapshots.

We can confirm that nothing's changed by re-running `lxc ls` and seeing that
there are still zero snapshots there:

``` console
$ lxc ls
+-------+---------+------+------+-----------+-----------+
| NAME  |  STATE  | IPV4 | IPV6 |   TYPE    | SNAPSHOTS |
+-------+---------+------+------+-----------+-----------+
| hello | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+
| world | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+
```

`--dry-run` is useful after you make some changes to the configuration and want
to confirm that everything is working as intended - since that's the case with
us, we can now re-run `lxc-snapper backup` without `--dry-run`:

``` console
$ lxd-snapper -c config.yaml backup

/* ... */

Summary
- processed instances: 2
- created snapshots: 2
```

... and voilà:

``` console
$ lxc ls
+-------+---------+------+------+-----------+-----------+
| NAME  |  STATE  | IPV4 | IPV6 |   TYPE    | SNAPSHOTS |
+-------+---------+------+------+-----------+-----------+
| hello | RUNNING | ...  | ...  | CONTAINER | 1         |
+-------+---------+------+------+-----------+-----------+
| world | RUNNING | ...  | ...  | CONTAINER | 1         |
+-------+---------+------+------+-----------+-----------+
```

Our policy says `keep-last: 2`, so let's go ahead and run `lxd-snapper backup`
twice more, to trigger this limit:

``` console
$ lxd-snapper -c config.yaml backup
$ lxd-snapper -c config.yaml backup

$ lxc ls
+-------+---------+------+------+-----------+-----------+
| NAME  |  STATE  | IPV4 | IPV6 |   TYPE    | SNAPSHOTS |
+-------+---------+------+------+-----------+-----------+
| hello | RUNNING | ...  | ...  | CONTAINER | 3         |
+-------+---------+------+------+-----------+-----------+
| world | RUNNING | ...  | ...  | CONTAINER | 3         |
+-------+---------+------+------+-----------+-----------+
```

Now we've got three snapshots per each container - why not two? Because as a
safety measure, the `backup` command always only _creates_ snapshots - never
deletes them.

To remove stale snapshots, you have to run `prune`:

``` console
$ lxd-snapper --dry-run -c config.yaml prune
Pruning instances:

- default/hello
-> keeping snapshot: auto-20180101-120200
-> keeping snapshot: auto-20180101-120100
-> deleting snapshot: auto-20180101-120000
-> [ OK ]

- default/world
-> keeping snapshot: auto-20180101-120200
-> keeping snapshot: auto-20180101-120100
-> deleting snapshot: auto-20180101-120000
-> [ OK ]

Summary
- processed instances: 2
- deleted snapshots: 2
- kept snapshots: 4
```

As before, we've started with `--dry-run` as to see if everything looks
alright - and since it seems so, it's time to kick those stale snapshots out of
our filesystem for good:

``` console
$ lxd-snapper -c config.yaml prune
/* ... */
Summary
- processed instances: 2
- deleted snapshots: 2
- kept snapshots: 4

$ lxc ls
+-------+---------+------+------+-----------+-----------+
| NAME  |  STATE  | IPV4 | IPV6 |   TYPE    | SNAPSHOTS |
+-------+---------+------+------+-----------+-----------+
| hello | RUNNING | ...  | ...  | CONTAINER | 2         |
+-------+---------+------+------+-----------+-----------+
| world | RUNNING | ...  | ...  | CONTAINER | 2         |
+-------+---------+------+------+-----------+-----------+
```

Re-running `prune` will now do nothing, since all of the containers have
correct number of snapshots:

``` console
$ lxd-snapper -c config.yaml prune
Pruning instances:

- default/hello
-> keeping snapshot: auto-20180101-120200
-> keeping snapshot: auto-20180101-120100
-> [ OK ]

- default/world
-> keeping snapshot: auto-20180101-120200
-> keeping snapshot: auto-20180101-120100
-> [ OK ]

Summary
- processed instances: 2
- deleted snapshots: 0
- kept snapshots: 4
```

_(there's also a command called `backup-and-prune` that does backup and prune
one after another - might come handy!)_

And that's basically it - that's how lxd-snapper works; now let's see what makes
it unique!

## Including and excluding instances

By default, each policy matches all instances across all [projects](https://ubuntu.com/tutorials/introduction-to-lxd-projects#1-overview) -
to affect that, you can use the `included-*` / `excluded-*` options:

``` yaml
policies:
  # Matches all instances inside the `client-a` project.
  a:
    included-projects: ['client-a']
    
  # Matches all instances _not_ inside the `client-`a project.
  b:
    excluded-projects: ['client-a']
    
  # Matches all instances named `container-a` across all projects.
  c:
    included-instances: ['container-a']
    
  # Matches all instances _not_ named `container-a` across all projects.
  d:
    excluded-instances: ['container-a']
    
  # Matches all instances that are running at the time of performing `backup` /
  # `prune`.
  #
  # Possible values: Aborting, Running, Starting, Stopped, and Stopping.
  e:
    included-statuses: ['Running']
    
  # Matches all instances that are _not_ running at the time of performing
  # `backup` / `prune`.
  f:
    excluded-statuses: ['Running']
    
  # Matches all instances named `php` or `nginx` that belong to project
  # `client-a` or `client-b`.
  #
  # For an instance to match this policy, it has to match all `included-*`
  # rules, so e.g.:
  #
  # - an instance named `php` for `client-c` will be skipped, since `client-c`
  #   doesn't match `included-projects`,
  #
  # - an instance named `nextcloud` for `client-a` will be skipped, since
  #   `nextcloud` doesn't match `included-instances`.
  #
  # In SQL, this would be:
  #
  # SELECT *
  #   FROM instances
  #  WHERE (project = "client-a" OR project = "client-b")
  #    AND (name = "php" OR name = "nginx")
  #    AND (status = "Running")
  g:
    included-projects: ['client-a', 'client-b']
    included-instances: ['php', 'nginx']
    included-statuses: ['Running']
 
  # Similarly as above (notice the reversed operator for `excluded-*`):
  #
  # SELECT *
  #   FROM instances
  #  WHERE (project = "client-a" OR project = "client-b")
  #    AND (name != "php" AND name != "nginx")
  h:
    included-projects: ['client-a', 'client-b']
    excluded-instances: ['php', 'nginx']
```

## Retention strategies

`keep-*` options determine for how long the snapshots should remain alive, so
we'd say that they define snapshots' **retention strategies**.

`keep-last` is the most straightforward setting, but it's not the only one -
lxd-snapper implements [Borg's approach](https://borgbackup.readthedocs.io/en/stable/usage/prune.html),
so one can get fancy:

``` yaml
policies:
  my-first-policy:
    keep-hourly: 6
    keep-daily: 5
    keep-weekly: 4
    keep-monthly: 3
    keep-yearly: 2
```

This would keep snapshots from 6 latest hours + 5 latest days + 4 latest weeks +
3 latest months + 2 latest years = 20 snapshots per instance.

Or, rephrasing:

- we'd have a snapshot per each past hour, up to 6 of them (e.g. 15:00, 14:00,
  13:00, 12:00, 11:00 & 10:00),
- we'd have a snapshot per each past day, up to 5 of them (e.g. today,
  yesterday, the day before yesterday, 3 days ago & 4 days ago),
- we'd have a snapshot per each past week, up to 4 of them (e.g. this week, the
  past week, two weeks ago & three weeks ago),
- et cetera, et cetera.

This system takes a while to get used to, but it's also extremely versatile;
you can find more examples inside the `docs/example-configs` directory and
inside [Borg's documentation](https://borgbackup.readthedocs.io/en/stable/usage/prune.html#examples).

Of course, this is all plug-and-play: if anything, `keep-last` should be enough
for typical use cases.

## Cascading

Say, you're using [LXD projects](https://ubuntu.com/tutorials/introduction-to-lxd-projects#1-overview)
and you've got a few containers:

``` console
$ lxc ls --project client-a
+-------+---------+------+------+-----------+-----------+
| NAME  |  STATE  | IPV4 | IPV6 |   TYPE    | SNAPSHOTS |
+-------+---------+------+------+-----------+-----------+
| mysql | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+
| php   | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+

$ lxc ls --project client-b
+-------+---------+------+------+-----------+-----------+
| NAME  |  STATE  | IPV4 | IPV6 |   TYPE    | SNAPSHOTS |
+-------+---------+------+------+-----------+-----------+
| mysql | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+
| php   | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+

$ lxc ls --project client-c
+-------+---------+------+------+-----------+-----------+
| NAME  |  STATE  | IPV4 | IPV6 |   TYPE    | SNAPSHOTS |
+-------+---------+------+------+-----------+-----------+
| mysql | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+
| php   | RUNNING | ...  | ...  | CONTAINER | 0         |
+-------+---------+------+------+-----------+-----------+
```

And, for the sake of argument, let's say that you want to create the following
configuration:

- all `mysql`-s should have 5 latest snapshots,
- all `php`-s should have 2 latest snapshots,
- except for `client-c`, which is important and should get 10 snapshots.

That's what cascading is for - when multiple policies match a single container:

``` yaml
policies:
  # Matches: client-a/mysql, client-b/mysql, client-c/mysql
  all-mysqls:
    included-instances: ['mysql']
    keep-last: 5
    
  # Matches: client-a/php, client-b/php, client-c/php
  all-phps:
    included-instances: ['php']
    keep-last: 2
    
  # Matches: client-c/mysql, client-c/php
  important-clients:
    included-projects: ['client-c']
    keep-last: 10
```

... lxd-snapper will combine them top-bottom into a single policy.

What this means practically is that when a few policies match a single
instance, policies that are below will have _higher priority_ than the ones
above them: `important-clients` is below `all-mysqls` and `all-phps`, so its
`keep-last` is more important for `client-c/mysql` and `client-c/php`.

This merging happens on a per-retention-strategy basis, so if we had:

``` yaml
policies:
  # Matches: client-a/mysql, client-b/mysql, client-c/mysql
  all-mysqls:
    included-instances: ['mysql']
    keep-daily: 2
    
  # Matches: client-a/php, client-b/php, client-c/php
  all-phps:
    included-instances: ['php']
    keep-hourly: 8
    
  # Matches: client-c/mysql, client-c/php
  important-clients:
    included-projects: ['client-c']
    keep-last: 20
```

... then our effective configuration would be:

``` 
client-a/mysql + client-b/mysql
  keep-daily = 2
  
client-a/php + client-b/php
  keep-hourly = 8

client-c/mysql
  keep-daily = 2
  keep-last = 20
  (= 22 snapshots)

client-c/php
  keep-hourly = 8
  keep-last = 20
  (= 28 snapshots)
```

Other possible use cases for this feature include creating a global "catch all"
policy, and then creating exceptions of it:

``` yaml
policies:
  all:
    keep-last: 10
    
  storages:
    include-containers: ['nextcloud', 'minio']
    keep-last: 20
```

This would keep 10 snapshots for all of the containers, with the exception of
`nextcloud` and `minio` that would have 20 snapshots.

## Hooks

Inside the configuration file, next to `policies:`, you can have a section
called `hooks` - they are small commands executed when a certain event inside
lxd-snapper happens:

``` yaml
hooks:
  on-backup-started: 'echo "on-backup-started" >> /tmp/log.txt'
  on-snapshot-created: 'echo "on-snapshot-created: {{projectName}}, {{instanceName}}, {{snapshotName}}" >> /tmp/log.txt'
  on-backup-completed: 'echo "on-backup-completed" >> /tmp/log.txt'

  on-prune-started: 'echo "on-prune-started" >> /tmp/log.txt'
  on-snapshot-deleted: 'echo "on-snapshot-deleted: {{projectName}}, {{instanceName}}, {{snapshotName}}" >> /tmp/log.txt'
  on-prune-completed: 'echo "on-prune-completed" >> /tmp/log.txt'

policies:
  # ...
```

Typical use cases include e.g. synchronizing snapshots to external storage:

``` yaml
hooks:
  on-snapshot-created: 'zfs send ... | ssh zfs recv ...'
  on-snapshot-deleted: 'zfs send ... | ssh zfs recv ...'

policies:
  # ...
```

As for the syntax, `{{something}}` is used for variable substitution -
lxd-snapper will replace e.g. `{{instanceName}}` with the snapshot's instance
name. There are three variables - `{{projectName}}`, `{{instanceName}}` and
`{{snapshotName}}` - and they are available only for the `on-snapshot-*` hooks.

You can provide at most one script per hook (i.e. you can't have
`on-backup-started` defined twice).

Hooks are run only from inside lxd-snapper (i.e. `on-snapshot-created` will not
be triggered for a manual `lxc snapshot` from the command line), and they are
skipped during `--dry-run`.

Hooks are run as soon as the event happens and block lxd-snapper until the hook
completes - e.g.:

``` yaml
hooks:
  on-snapshot-created: 'delay 10'
```

... will delay creating _each_ snapshot by 10 seconds; if that's problematic for
your use case, you might want to buffer the changes like so:

``` yaml
hooks:
  on-backup-started: 'rm /tmp/created-snapshots.txt'
  on-snapshot-created: 'echo "{{instanceName}},{{snapshotName}}" >> /tmp/created-snapshots.txt'
  on-backup-completed: './sync-snapshots.sh /tmp/created-snapshots.txt'
```

## Scheduling

Finally, lxd-snapper is a fire-and-forget application - it doesn't daemonize
itself; to keep instances backed-up & pruned on time, you'll most likely want to
create a systemctl timer or a cronjob for it:

```
5 * * * * /usr/bin/lxd-snapper -c /etc/lxd-snapper.yaml backup-and-prune
```

# Configuration syntax

```yaml
snapshot-name-prefix: 'auto-'

hooks:
  on-backup-started: '...'
  on-snapshot-created: '...'
  on-backup-completed: '...'
  on-prune-started: '...'
  on-snapshot-deleted: '...'
  on-prune-completed: '...'

policies:
  policy-name:
    included-projects: ['...', '...']
    excluded-projects: ['...', '...']
    included-instances: ['...', '...']
    excluded-instances: ['...', '...']
    included-statuses: ['...', '...']
    excluded-statuses: ['...', '...']

    keep-hourly: 1
    keep-daily: 1
    keep-weekly: 1
    keep-monthly: 1
    keep-yearly: 1
    keep-last: 1
    keep-limit: 1
```

# Contributing

Merge requests are very much welcome! :-)

lxd-snapper is a pretty standard Rust project, so cargo & rustc should be enough
to get you going; there are also end-to-end tests written using [NixOS Testing
Framework](https://nix.dev/tutorials/integration-testing-using-virtual-machines)
that you can run with `nix flake check` (requires
<https://nixos.wiki/wiki/Flakes>).

# Disclaimer

Snapshots are _not_ a replacement for backups - to keep your data safe, use
snapshots and backups together, wisely.

# License

Copyright (c) 2019-2022, Patryk Wychowaniec <pwychowaniec@pm.me>.    
Licensed under the MIT license.
