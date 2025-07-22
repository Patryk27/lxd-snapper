# Upcoming

- Added support for Incus.
- Dropped support for LXD 4 and LXD 5.

# 1.3.1

- New configuration option: `lxc-timeout`.
- Fixed a typo where lxd-snapper would say `Error::` instead of just `Error:`.

# 1.3.0

- lxd-snapper now supports LXD remotes! 
- New hooks: `on_instance_backed_up`, `on_instance_pruned`.
- Revamped output to be more user-friendly.
- Required rustc version is now 1.63.0.
- Dropped support for i686-linux.
