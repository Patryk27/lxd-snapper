machine.succeed("lxc launch image test")


with subtest("Backup"):
    assert_snapshot_does_not_exist("default", "test", "auto\-.*")

    out = lxd_snapper("backup")

    assert (
        "created snapshots: 1" in out
    ), f"created snapshots != 1; actual output: {out}"

    assert_snapshot_exists("default", "test", "auto\-.*")


with subtest("Prune"):
    assert_snapshot_exists("default", "test", "auto\-.*")

    out = lxd_snapper("prune")

    assert (
        "deleted snapshots: 1" in out
    ), f"deleted snapshots != 1; actual output: {out}"

    assert_snapshot_does_not_exist("default", "test", "auto\-.*")
