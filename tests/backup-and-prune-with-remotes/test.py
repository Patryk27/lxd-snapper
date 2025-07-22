main = MyMachine(main)
serverA = MyMachine(serverA)
serverB = MyMachine(serverB)
serverC = MyMachine(serverC)

def setup(server, name, ip):
    server.succeed("date -s '2018-01-01 13:00:00'")
    server.succeed("lxc-or-incus config set core.https_address 0.0.0.0")

    if main.flavor() == "lxd":
        server.succeed("lxc config set core.trust_password test")
        main.succeed(f"lxc remote add {name} {ip} --accept-certificate --password test")
    else:
        token = server.succeed("incus config trust add main -q")
        main.succeed(f"incus remote add {name} {token}")

# Setup servers
setup(serverA, "serverA", "192.168.1.2")
setup(serverB, "serverB", "192.168.1.3")
setup(serverC, "serverC", "192.168.1.4")

# Create containers
main.succeed("lxc-or-incus launch image container")
serverA.succeed("lxc-or-incus launch image container")
serverB.succeed("lxc-or-incus launch image container")
serverC.succeed("lxc-or-incus launch image container")

# Backup containers
main.lxd_snapper("backup", "expected.out.1.txt")
main.assert_snapshot_count("default", "container", ".*", 0)
serverA.assert_snapshot_count("default", "container", ".*", 1)
serverB.assert_snapshot_count("default", "container", ".*", 0)
serverC.assert_snapshot_count("default", "container", ".*", 1)

# Prune containers
main.lxd_snapper("prune", "expected.out.2.txt")
main.assert_snapshot_count("default", "container", ".*", 0)
serverA.assert_snapshot_count("default", "container", ".*", 0)
serverB.assert_snapshot_count("default", "container", ".*", 0)
serverC.assert_snapshot_count("default", "container", ".*", 0)
