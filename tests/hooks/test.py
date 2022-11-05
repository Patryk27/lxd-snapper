machine = MyMachine(machine)
machine.succeed("lxc launch image test")
machine.succeed("touch /tmp/log.txt")

# ---

machine.lxd_snapper("--dry-run backup", "expected.out.1.txt")
machine.lxd_snapper("--dry-run prune", "expected.out.2.txt")

actual_log = machine.succeed("cat /tmp/log.txt")

assert "" == actual_log, f"hook-log should be empty, but it isn't:\n{actual_log}"

# ---

machine.lxd_snapper("backup", "expected.out.3.txt")
machine.lxd_snapper("prune", "expected.out.4.txt")

actual_log = machine.succeed("cat /tmp/log.txt")
expected_log = machine.succeed("cat /test/expected.log.txt")

assert expected_log == actual_log, f"hook-logs don't match; actual:\n{actual_log}"
