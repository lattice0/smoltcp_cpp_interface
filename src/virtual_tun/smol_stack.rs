use smoltcp_openvpn_bridge::virtual_tun::VirtualTunInterface;
use smoltcp::phy::{self, sys, DeviceCapabilities, Device};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer, SocketBuffer};

pub struct SmolStack {
    device: Device,
    sockets: Set<'a, 'b: 'a, 'c: 'a + 'b>,
    interface: Interface<'b, 'c, 'e, Device>
}

impl SmolStack {
    pub fn new(interface_name: String) -> Result<()> {
        let device = VirtualTunInterface::new("tun1").unwrap();
        rx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
        tx_buffer = TcpSocketBuffer::new(vec![0; 1024]);
        
    }

    pub fn add_socket() -> Self {
        let rx_buffer = SocketBuffer,
        let tx_buffer = SocketBuffer,
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
    }

    pub fn add_ipv4_address(cidr: Ipv4Cidr) -> Self {

    }

    pub fn add_ipv6_address(cidr: Ipv6Cidr) -> Self {

    }

    pub fn add_default_v4_gateway(ipv4_address: Ipv4Address) -> Self {

    }

    pub fn add_default_v6_gateway(ipv4_address: Ipv6Address) -> Self {
        
    }
}