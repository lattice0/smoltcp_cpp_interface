ip tuntap add dev tun1 mode tun user `id -un`
ip link set dev tun1 up
ip addr add dev tun1 local 192.168.69.0 remote 192.168.69.1
iptables -t filter -I FORWARD -i tun1 -o eth0 -j ACCEPT
iptables -t filter -I FORWARD -m state --state ESTABLISHED,RELATED -j ACCEPT
iptables -t nat -I POSTROUTING -o eth0 -j MASQUERADE
sysctl net.ipv4.ip_forward=1