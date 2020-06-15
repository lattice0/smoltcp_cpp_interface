#include <iostream>
#include "interface.h"

extern "C" void cppDeleteArray(uint8_t *data) {
    delete[] data;
}

extern "C" void cppDeletePointer(uint8_t *data) {
    delete data;
}


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
