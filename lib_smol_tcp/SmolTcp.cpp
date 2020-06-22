#include <iostream>
#include "interface.h"
#include <map>

int main()
{
    HandleMap<SmolSocket> smolSockethandleMap;
    TunSmolStack tunSmolStack("tun0");

    tunSmolStack.addIpv4Address(CIpv4Cidr{
        CIpv4Address{
            {192, 168, 69, 1}},
        24});

    tunSmolStack.addIpv6Address(CIpv6Cidr{
        CIpv6Address{
            {0xfdaa, 0, 0, 0, 0, 0, 0, 1}},
        64});
    tunSmolStack.addIpv6Address(CIpv6Cidr{
        CIpv6Address{
            {0xfe80, 0, 0, 0, 0, 0, 0, 1}},
        64});

    tunSmolStack.addDefaultV4Gateway(CIpv4Address{
        {192, 168, 69, 100}});

    tunSmolStack.addDefaultV6Gateway(CIpv6Address{
        {0xfe80, 0, 0, 0, 0, 0, 0, 0x100}});

    SocketHandleKey socketHandle = tunSmolStack.addSocket(SOCKET_TCP);
    SmolSocket smolSocket;
    smolSocket.SocketHandleKey = socketHandle;
    size_t smolSocketHandle = smolSockethandleMap.emplace(smolSocket);
    uint8_t result = tunSmolStack.finalize();
    CIpEndpoint endpointNone{
        CIpEndpointType::None,
        CIpv4Address{},
        CIpv6Address{},
        0};
    enum State
    {
        Connect,
        Request,
        Response
    };
    State state = State::Connect;

    if (result == 0)
    {
        //socketLoop(tunSmolStack, handle);
        while (true)
        {
            tunSmolStack.poll();
            if (state == State::Connect)
            {
                uint16_t randomOutputPort = tunSmolStack.randomOutputPort();
                tunSmolStack.connectIpv4(CIpv4Address{
                                             {192, 168, 69, 1}},
                                         randomOutputPort, 80);
                state = State::Request;
            }
            if (state == State::Request)
            {
                std::string httpRequestData("GET /hello.htm HTTP/1.1\r\n\
                    Host: www.tutorialspoint.com\r\n\
                    Connection: Keep-Alive\r\n\
                    \r\n");
                const uint8_t* httpRequestDataBuffer = reinterpret_cast<const uint8_t*>(httpRequestData.c_str());
                tunSmolStack.send(smolSocketHandle, httpRequestDataBuffer, httpRequestData.size(), endpointNone);
                state = State::Response;
            }
            if (state == State::Response)
            {
                
            }
            tunSmolStack.spin(smolSocketHandle);
        }
    }
    else
    {
        //throw
        std::cout << "error on finalize" << std::endl;
    }

    getchar();
    return 0;
}
