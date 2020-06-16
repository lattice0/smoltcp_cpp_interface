#include <iostream>
#include "interface.h"

int main() {
    
    CIpv6Cidr cIpv6Cidr1 {
        CIpv6Address {
            {0xfdaa, 0, 0, 0, 0, 0, 0, 1}
        },
        64
    };
    
    getchar();
    return 0;
}
