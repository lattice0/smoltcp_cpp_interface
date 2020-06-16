cargo build
clang++ -shared -fPIC -o libsmoltcp_openvpn_bridge.so -L target/debug/ src/virtual_tun/interface.cpp -lstdc++ -lsmoltcp_openvpn_bridge_rust -pthread -ldl
clang++ -o smoltcp_example -L . lib_smol_tcp/*.cpp -L target/debug/ -I src/virtual_tun -lsmoltcp_openvpn_bridge
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$PWD:$PWD/target/debug