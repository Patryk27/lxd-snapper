machine = MyMachine(machine)
machine.succeed("lxc launch image test")
machine.succeed("touch /tmp/log.txt")

# ---

machine.lxd_snapper("--dry-run backup", "expected.out.1.txt")
machine.lxd_snapper("--dry-run prune", "expected.out.2.txt")

actual_log = machine.succeed("cat /tmp/log.txt")
expected_log = ""

assert expected_log == actual_log, f"logs don't match; actual log:\n{actual_log}"

# ---

machine.lxd_snapper("backup", "expected.out.3.txt")
machine.lxd_snapper("prune", "expected.out.4.txt")

actual_log = machine.succeed("cat /tmp/log.txt")
expected_log = machine.succeed("cat /test/expected.log.txt")

assert expected_log == actual_log, f"logs don't match; actual log:\n{actual_log}"
