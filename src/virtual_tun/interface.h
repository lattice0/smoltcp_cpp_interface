#ifndef SMOL_TCP_INTERFACE_H
#define SMOL_TCP_INTERFACE_H

typedef void* TunSmolStackPtr;
typedef void* SocketHandlePtr;

static int SOCKET_TCP = 0;
static int SOCKET_UDP = 1;

struct CIpv4Address {
    uint8_t address[4];
};

struct CIpv6Address {
    uint16_t address[8];
};

struct CIpv4Cidr {
    CIpv4Address address;
    uint32_t prefix;
};

struct CIpv6Cidr {
    CIpv6Address address;
    uint64_t prefix;
};

extern "C" void cppDeleteArray(uint8_t *data) {
    delete[] data;
}

extern "C" void cppDeletePointer(uint8_t *data) {
    delete data;
}

extern "C" TunSmolStackPtr smol_stack_tun_smol_stack_builder_new(const char* interfaceName);
extern "C" SocketHandlePtr smol_stack_add_socket(TunSmolStackPtr, uint8_t);
extern "C" void smol_stack_spin(TunSmolStackPtr, SocketHandlePtr);
extern "C" void smol_stack_add_ipv4_address(TunSmolStackPtr, CIpv4Cidr);
extern "C" void smol_stack_add_ipv6_address(TunSmolStackPtr, CIpv6Cidr);
extern "C" void smol_stack_add_default_v4_gateway(TunSmolStackPtr, CIpv4Address);
extern "C" void smol_stack_add_default_v6_gateway(TunSmolStackPtr, CIpv6Address);
extern "C" uint8_t smol_stack_finalize(TunSmolStackPtr);

class TunSmolStack {
private:
    TunSmolStackPtr tunSmolStackPtr;
public:
    TunSmolStack(std::string interfaceName) {
        tunSmolStackPtr = smol_stack_tun_smol_stack_builder_new(interfaceName.c_str());
    }

    SocketHandlePtr addSocket(uint8_t socketType) {
        return smol_stack_add_socket(tunSmolStackPtr, socketType);
    }

    void spin(SocketHandlePtr socketHandlePtr) {
        smol_stack_spin(tunSmolStackPtr, socketHandlePtr);
    }

    void addIpv4Address(CIpv4Cidr cidr) {
        smol_stack_add_ipv4_address(tunSmolStackPtr, cidr);
    }

    void addIpv6Address(CIpv6Cidr cidr) {
        smol_stack_add_ipv6_address(tunSmolStackPtr, cidr);
    }

    void addDefaultV4Gateway(CIpv4Address address) {
        smol_stack_add_default_v4_gateway(tunSmolStackPtr, address);
    }

    void addDefaultV6Gateway(CIpv6Address address) {
        smol_stack_add_default_v6_gateway(tunSmolStackPtr, address);
    }

    uint8_t finalize() {
        return smol_stack_finalize(tunSmolStackPtr);
    }
};

#endif //SMOL_TCP_INTERFACE_H