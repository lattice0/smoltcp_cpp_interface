#ifndef SMOL_TCP_INTERFACE_H
#define SMOL_TCP_INTERFACE_H

typedef void* TunSmolStack;
typedef void* SocketHandle;

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

extern "C" TunSmolStack smol_stack_tun_smol_stack_builder_new(const char* interfaceName);
extern "C" SocketHandle smol_stack_add_socket(TunSmolStack, uint8_t);
extern "C" void smol_stack_spin(TunSmolStack, SocketHandle);
extern "C" void smol_stack_add_ipv4_address(TunSmolStack, CIpv4Cidr);
extern "C" void smol_stack_add_ipv6_address(TunSmolStack, CIpv6Cidr);
extern "C" void smol_stack_add_default_v4_gateway(TunSmolStack, CIpv4Address);
extern "C" void smol_stack_add_default_v6_gateway(TunSmolStack, CIpv6Address);
extern "C" uint8_t smol_stack_finalize(TunSmolStack);

class TunSmolStackBuilder {
private:
    TunSmolStack tunSmolStackPtr;
public:
    TunSmolStackBuilder(std::string interfaceName) {
        tunSmolStackPtr = smol_stack_tun_smol_stack_builder_new(interfaceName.c_str());
    }

    void addIpv4Address(CIpv4Cidr cidr) {
        smol_stack_add_ipv4_address(tunSmolStackPtr, cidr);
    }

    void addIpv4Address(CIpv6Cidr cidr) {
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