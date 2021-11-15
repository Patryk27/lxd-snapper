machine.succeed("lxc project switch default")
machine.succeed("lxc launch alpine test")

with subtest("Backup"):
    machine.succeed("date -s '2018-01-01 12:00:00'")

    out = run("backup")

    assert (
        "created snapshots: 1" in out
    ), f"created snapshots != 1; actual output: {out}"

with subtest("Backup (dry-run)"):
    machine.succeed("date -s '2018-01-01 12:01:00'")

    run("--dry-run backup")

with subtest("Prune"):
    out = run("prune")

    assert (
        "deleted snapshots: 1" in out
    ), f"deleted snapshots != 1; actual output: {out}"

with subtest("Prune (dry-run)"):
    run("--dry-run prune")

actual_log = machine.succeed("cat /tmp/log.txt")
expected_log = machine.succeed("cat /test/expected.log.txt")

assert expected_log == actual_log, f"logs don't match; actual log: {actual_log}"
