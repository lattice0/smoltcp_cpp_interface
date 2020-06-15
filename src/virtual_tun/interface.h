#ifndef INTERFACE_H
#define INTERFACE_H

typedef void* TunSmolStackBuilderPtr;

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

extern "C" TunSmolStackBuilderPtr smol_stack_tun_smol_stack_builder_new(char* interfaceName);
extern "C" void smol_stack_add_ipv4_address(TunSmolStackBuilderPtr tunSmolStackBuilderPtr, cidr: CIpv4Cidr);
extern "C" void smol_stack_add_ipv6_address(TunSmolStackBuilderPtr tunSmolStackBuilderPtr, cidr: CIpv6Cidr);
extern "C" void smol_stack_add_default_v4_gateway(TunSmolStackBuilderPtr tunSmolStackBuilderPtr, address: CIpv4Address);
extern "C" void smol_stack_add_default_v6_gateway(TunSmolStackBuilderPtr tunSmolStackBuilderPtr, address: CIpv6Address);
extern "C" uint8_t smol_stack_finalize(TunSmolStackBuilderPtr tunSmolStackBuilderPtr);

class TunSmolStackBuilder {
private:
    TunSmolStackBuilderPtr tunSmolStackBuilderPtr;
public:
    TunSmolStackBuilder(std::string interfaceName) {
        tunSmolStackBuilderPtr = smol_stack_tun_smol_stack_builder_new(interfaceName.c_str());
    }

    void addIpv4Address(CIpv4Cidr cidr) {
        smol_stack_add_ipv4_address(tunSmolStackBuilderPtr, cidr);
    }

    void addIpv4Address(CIpv6Cidr cidr) {
        smol_stack_add_ipv6_address(tunSmolStackBuilderPtr, cidr);
    }

    void addDefaultV4Gateway(CIpv4Address address) {
        smol_stack_add_default_v4_gateway(tunSmolStackBuilderPtr, address);
    }

    void addDefaultV6Gateway(CIpv6Address address) {
        smol_stack_add_default_v6_gateway(tunSmolStackBuilderPtr, address);
    }

    uint8_t finalize() {
        return smol_stack_finalize(tunSmolStackBuilderPtr);
    }
}

#endif //INTERFACE_H