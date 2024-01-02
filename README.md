# lxd-snapper

**LXD snapshots, automated.**

lxd-snapper automates creating & removing LXD snapshots - just prepare a
snapshotting policy, setup a cronjob, and enjoy your containers.

tl;dr it's a fancy wrapper for `lxc snapshot` & `lxc delete`; like LXD's
built-in `snapshots.schedule`, but more powerful.

# Requirements

- LXD 4 / 5
- Linux (x86_64)

Plus, if you plan on building lxd-snapper locally:

- Cargo & Rust 1.63.0
- Nix (for running integration tests)

# Getting started

## Downloading

You can either download pre-built binaries:

``` console
# x86_64
$ wget https://github.com/Patryk27/lxd-snapper/releases/download/v1.3.0/lxd-snapper-linux64 -O lxd-snapper
$ chmod u+x lxd-snapper
```

... or build lxd-snapper on your own:

``` console
$ git clone https://github.com/Patryk27/lxd-snapper
$ cd lxd-snapper

# Using Cargo
$ cargo build --release
$ ./target/release/lxd-snapper

# (or) Using Nix v3
$ nix build
$ ./result/bin/lxd-snapper
```

## Configuring

Setting-up lxd-snapper is easy: you just need to prepare a configuration file
that will describe which LXD instances (so containers and/or virtual machines)
you want to get snapshotted and for how long those snapshots should be kept
around.

We can start with the most basic configuration:

``` yaml
policies:
  my-first-policy:
    keep-last: 2
```

... which defines a single policy called `my-first-policy` that will snapshot
all of your instances, keeping around the latest two snapshots per each
instance.

To check how it works, let's go ahead and create some containers:

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

Now, to snapshot those containers, first we have to store that configuration
from before into a file - say, `config.yaml` - and then run `lxd-snapper
backup`:

``` console
$ lxd-snapper --dry-run -c config.yaml backup
(--dry-run is active, no changes will be applied)

hello
  - creating snapshot: auto-20221105-130019 [ OK ]

world
  - creating snapshot: auto-20221105-130019 [ OK ]

Summary
-------
  processed instances: 2
  created snapshots: 2
```

As you can see, there's a detailed output of everything that's happened - or
rather of everything that _would_ happen: we used a switch called `--dry-run`
which tells lxd-snapper that you only want to **preview** the changes without
actually creating or removing any snapshots.

We can confirm that nothing's changed by re-running `lxc ls` and seeing that
we've still got zero snapshots:

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

`--dry-run` is useful after you've made some changes to the configuration and
want to confirm that everything is working as intended - since that's the case
with us, we can now re-run `lxc-snapper backup` without `--dry-run`:

``` console
$ lxd-snapper -c config.yaml backup

/* ... */

Summary
-------
  processed instances: 2
  created snapshots: 2
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

To remove stale snapshots, we have to run `prune`:

``` console
$ lxd-snapper --dry-run -c config.yaml prune
(--dry-run is active, no changes will be applied)

hello
  - keeping snapshot: auto-20221105-130214
  - keeping snapshot: auto-20221105-130213
  - deleting snapshot: auto-20221105-130157 [ OK ]

world
  - keeping snapshot: auto-20221105-130214
  - keeping snapshot: auto-20221105-130213
  - deleting snapshot: auto-20221105-130157 [ OK ]

Summary
-------
  processed instances: 2
  deleted snapshots: 2
  kept snapshots: 4
```

As before, we've started with `--dry-run` as to see if everything looks
alright - and since it seems so, it's time to kick those stale snapshots out of
our filesystem for good:

``` console
$ lxd-snapper -c config.yaml prune

/* ... */

Summary
-------
  processed instances: 2
  deleted snapshots: 2
  kept snapshots: 4

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
hello
  - keeping snapshot: auto-20221105-130214
  - keeping snapshot: auto-20221105-130213

world
  - keeping snapshot: auto-20221105-130214
  - keeping snapshot: auto-20221105-130213

Summary
-------
  processed instances: 2
  deleted snapshots: 0
  kept snapshots: 4
```

_(there's also a command called `backup-and-prune` that runs backup and prune
one after another, which is what you'll usually want to do.)_

And that's basically it - that's how lxd-snapper works; now let's see what makes
it unique!

## Filtering instances

By default, lxd-snapper snapshots all of the instances it can find on the local
machine - you can affect that with various `included-` and `excluded-` options:

``` yaml
policies:
  # Matches all instances inside the `important-client` project and keeps the
  # last 20 snapshots for each of them:
  a:
    included-projects: ['important-client']
    keep-last: 20
    
  # Matches all instances _outside_ the `important-client` project and keeps the
  # last 5 snapshots for each of them:
  b:
    excluded-projects: ['important-client']
    keep-last: 5
    
  # Matches all instances named `important-container` (across all projects) and
  # keeps the last 20 snapshots for each of them:
  c:
    included-instances: ['important-container']
    keep-last: 20
    
  # Matches all instances _not_ named `important-container` (across all
  # projects) and keeps the last 5 snapshots for each of them:
  d:
    excluded-instances: ['important-container']
    keep-last: 5
    
  # Matches all instances that are running at the time of performing `backup` /
  # `prune`.
  #
  # Possible values: Aborting, Running, Ready, Starting, Stopped, and Stopping.
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

lxd-snapper supports [Borg-style](https://borgbackup.readthedocs.io/en/stable/usage/prune.html)
retention strategies; each policy must specify at least one `keep-` option that
says for how long its snapshots should be kept around.

The most straightforward setting is `keep-last` - e.g.:

``` yaml
policies:
  my-policy:
    keep-last: 5
```

... would keep the five _newest_ snapshots for each container.

(i.e. if you ran `backup-and-prune` once a day, that would effectively keep the
five days worth of snapshots around)

Being versatile, lxd-snapper also supports `keep-hourly`, `keep-daily` etc.,
allowing you to create more fancy policies such as:

``` yaml
policies:
  my-policy:
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

Of course, you don't have to get fancy -- `keep-last` should get the job done
most of the time.

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

... lxd-snapper will combine them top-bottom into a single policy, separately
for each container.

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

Hooks are small shell commands executed when lxd-snapper performs a certain
action; you can configure them by creating a `hooks:` section inside the
configuration:

``` yaml
hooks:
  on-backup-started: 'echo "on-backup-started" >> /tmp/log.txt'
  on-snapshot-created: 'echo "on-snapshot-created: {{ remoteName }}, {{ projectName }}, {{ instanceName }}, {{snapshotName}}" >> /tmp/log.txt'
  on-instance-backed-up: 'echo "on-instance-backed-up: {{ remoteName }}, {{ projectName }}, {{ instanceName }}" >> /tmp/log.txt'
  on-backup-completed: 'echo "on-backup-completed" >> /tmp/log.txt'

  on-prune-started: 'echo "on-prune-started" >> /tmp/log.txt'
  on-snapshot-deleted: 'echo "on-snapshot-deleted: {{ remoteName }}, {{ projectName }}, {{ instanceName }}, {{ snapshotName }}" >> /tmp/log.txt'
  on-instance-pruned: 'echo "on-instance-pruned: {{ remoteName }}, {{ projectName }}, {{ instanceName }}" >> /tmp/log.txt'
  on-prune-completed: 'echo "on-prune-completed" >> /tmp/log.txt'

policies:
  # ...
```

They come handy e.g. for synchronizing snapshots to external storage:

``` yaml
hooks:
  on-snapshot-created: 'zfs send ... | ssh zfs recv ...'
  on-snapshot-deleted: 'zfs send ... | ssh zfs recv ...'

policies:
  # ...
```

Most of the hooks support _variable interpolation_ - they are strings that are
replaced by lxd-snapper with some concrete value before the hook is run:

- `on-snapshot-created` has `{{ remoteName }}`, `{{ projectName }}`, `{{ instanceName }}` and `{{ snapshotName }}`,
- `on-instance-backed-up` has `{{ remoteName }}`, `{{ projectName }}` and `{{ instanceName }}`,
- `on-snapshot-deleted` has `{{ remoteName }}`, `{{ projectName }}`, `{{ instanceName }}` and `{{ snapshotName }}`,
- `on-instance-pruned` has `{{ remoteName }}`, `{{ projectName }}` and `{{ instanceName }}`.

\... where:

- `{{ remoteName }}` corresponds to `NAME` as visible in `lxc remote ls`
  (`local` by default),
- `{{ projectName }}` corresponds to `NAME` as visible in `lxc project ls`
  (`default` by default),
- `{{ instanceName }}` corresponds to `NAME` as visible in `lxc ls`,
- `{{ snapshotName }}` corresponds to `NAME` as visible in `lxc info instance-name`.

Caveats & Tips:

- hooks are skipped during `--dry-run`,

- you can provide at most one script per hook (e.g. you can't have
  `on-backup-started` defined twice),
  
- you don't have to provide scripts for hooks you're not interested in (e.g.
  specifying just `on-backup-started` is alright),

- hooks are run only from inside lxd-snapper (e.g. `on-snapshot-created` will
  not be run for a manual `lxc snapshot` performed from the command line), 

- hooks are launched as soon as the event happens and block lxd-snapper until
  the hook completes - e.g.

  ``` yaml
  hooks:
    on-snapshot-created: 'delay 10'
  ```

  ... will delay creating _each_ snapshot by 10 seconds; if that's problematic
  for your use case, you might want to buffer the changes like so:

  ``` yaml
  hooks:
    on-backup-started: 'rm /tmp/created-snapshots.txt'
    on-snapshot-created: 'echo "{{ instanceName }},{{ snapshotName }}" >> /tmp/created-snapshots.txt'
    on-backup-completed: './sync-snapshots.sh /tmp/created-snapshots.txt'
  ```

- when a hook returns a non-zero exit code, it will be treated as an error,

- hook's stdout and stderr are not displayed, unless the hook returns a non-zero
  exit code (stdout & stderr will be then visible in the error message),
  
- variables can be written `{{likeThat}}` or `{{ likeThat }}`, whichever way you
  prefer.

## Remotes

By default, lxd-snapper sees containers & virtual machines only from the local
LXD instance (i.e. as if you run `lxc ls`).

If you're using LXD remotes, and you'd like for lxd-snapper to snapshot them
too, you have to provide their names in the configuration file:

``` yaml
remotes:
  - server-a
  - server-b
  - server-c
```

If you'd like to snapshot both the local LXD _and_ the remote ones, use a remote
called `local`:

``` yaml
remotes:
  - local
  - server-a
  - server-b
  - server-c
```

(those labels correspond to `NAME` as visible in `lxc remote ls`)

By default, each policy will match all of the specified remotes - if you want to
narrow that down, you can use `included-remotes` and `excluded-remotes`:

``` yaml
remotes:
  - unimportant-server-A
  - unimportant-server-B
  - important-server-A

policies:
  all-servers:
    keep-last: 10
  
  important-servers:
    included-remotes: ['important-server-A']
    keep-last: 25 
```

If you're going for a centralized backup solution, you can pair this feature 
with _hooks_ to pull the newly-created snapshots into your coordinator-machine:

``` yaml
hooks:
  on-instance-backed-up: 'lxc copy --refresh {{ remoteName }}:{{ instanceName }} {{ instanceName }}'
  on-instance-pruned: 'lxc copy --refresh {{ remoteName }}:{{ instanceName }} {{ instanceName }}'

remotes:
  - server-A
  - server-B
  - server-C

policies:
  all-servers:
    keep-last: 10
```

## Scheduling

Finally, lxd-snapper is a fire-and-forget application - it doesn't daemonize
itself; to keep instances backed-up & pruned on time, you will want to create a
systemctl timer or a cronjob for it:

```
5 * * * * /usr/bin/lxd-snapper -c /etc/lxd-snapper.yaml backup-and-prune
```

# Configuration syntax reference

```yaml
# (optional, defaults to 'auto-')
#
# Prefix used to distinguish between snapshots created by lxd-snapper and 
# everything else (e.g. a manual `lxc snapshot`).
#
# `lxd-snapper backup` will create snapshots with this prefix and
# `lxd-snapper prune` will only ever remove snapshots that match this prefix.
snapshot-name-prefix: '...'

# (optional, defaults to '%Y%m%d-%H%M%S')
#
# Formatting string used to build the rest of the snapshot name.
# 
# Format:
# https://docs.rs/chrono/0.4.22/chrono/format/strftime/index.html
snapshot-name-format: '...'

# (optional, defaults to '10m')
#
# Timeout for each call to lxc; prevents lxd-snapper from running forever if lxc
# happens to hang.
#
# If you've got a (very) slow storage, you might want to increase this limit, 
# but the default should be enough for a typical setup.
#
# Format:
# https://docs.rs/humantime/latest/humantime/
# (e.g. '30s', '5m', '1h' etc.)
lxc-timeout: '...'

# (optional)
hooks:
  on-backup-started: '...'
  on-instance-backed-up: '...'
  on-snapshot-created: '...'
  on-backup-completed: '...'
  
  on-prune-started: '...'
  on-snapshot-deleted: '...'
  on-instance-pruned: '...'
  on-prune-completed: '...'

# (optional, defaults to `local`)
remotes:
  - local
  - server-A
  - server-B

# (at least one required)
policies:
  policy-name:
    included-remotes: ['...', '...']
    excluded-remotes: ['...', '...']
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
to get you going.

There are also end-to-end tests written using [NixOS Testing Framework](https://nix.dev/tutorials/integration-testing-using-virtual-machines)
that you can run with `nix flake check -j4`.

# Disclaimer

Snapshots are _not_ a replacement for backups - to keep your data safe, use
snapshots and backups together, wisely.

# License

Copyright (c) 2019-2024, Patryk Wychowaniec <pwychowaniec@pm.me>.    
Licensed under the MIT license.
