{ fw }:

let
  mkServerNode = ip: fw.mkNode {
    networking = {
      firewall = {
        allowedTCPPorts = [ 8443 ];
      };

      interfaces = {
        eth1 = {
          ipv4 = {
            addresses = [
              { address = ip; prefixLength = 24; }
            ];
          };
        };
      };
    };
  };

in
fw.mkTest {
  nodes = {
    main = fw.mkNode { };
    serverA = mkServerNode "192.168.1.2";
    serverB = mkServerNode "192.168.1.3";
    serverC = mkServerNode "192.168.1.4";
  };
}
