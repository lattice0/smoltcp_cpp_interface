cargo build
clang++ -shared -fPIC -o smoltcp_openvpn_bridge -L target/debug/ src/virtual_tun/interface.cpp -lstdc++ -lsmoltcp_openvpn_bridge_rust -pthread -ldl