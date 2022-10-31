machine.succeed("date -s '2018-01-01 12:00:00'")
machine.succeed("lxc launch image test")
machine.succeed("touch /tmp/log.txt")


with subtest("Dry run"):
    lxd_snapper("--dry-run backup", "/test/expected.out.1.txt")
    lxd_snapper("--dry-run prune", "/test/expected.out.2.txt")

    actual_log = machine.succeed("cat /tmp/log.txt")
    expected_log = ""

    assert expected_log == actual_log, f"logs don't match; actual log:\n{actual_log}"


with subtest("Actual run"):
    lxd_snapper("backup", "/test/expected.out.3.txt")
    lxd_snapper("prune", "/test/expected.out.4.txt")

    actual_log = machine.succeed("cat /tmp/log.txt")
    expected_log = machine.succeed("cat /test/expected.log.txt")

    assert expected_log == actual_log, f"logs don't match; actual log:\n{actual_log}"
