machine = MyMachine(machine)
machine.succeed("lxc launch image test")
machine.lxd_snapper_err("backup", "expected.out.txt")
