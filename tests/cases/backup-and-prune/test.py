machine.succeed("lxc project switch default")
machine.succeed("lxc launch alpine test")

with subtest("Backup"):
    assert_snapshot_does_not_exist("default", "test", "auto\-.*")

    out = run("backup")

    assert (
        "created snapshots: 1" in out
    ), f"created snapshots != 1; actual output: {out}"

    assert_snapshot_exists("default", "test", "auto\-.*")

with subtest("Prune"):
    assert_snapshot_exists("default", "test", "auto\-.*")

    out = run("prune")

    assert (
        "deleted snapshots: 1" in out
    ), f"deleted snapshots != 1; actual output: {out}"

    assert_snapshot_does_not_exist("default", "test", "auto\-.*")
