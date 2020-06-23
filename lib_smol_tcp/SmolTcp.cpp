#include <iostream>
#include "interface.h"
#include <map>

int main()
{
    HandleMap<SmolSocket> smolSockethandleMap;
    TunSmolStack tunSmolStack("tun1", StackType::Tun);

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
    std::cout << "smol socket handle key " << socketHandle << std::endl;
    size_t smolSocketHandle = smolSockethandleMap.emplace(smolSocket);
    std::cout << "smolSocketHandle " << socketHandle << std::endl;
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
                std::cout << "connecting..." << std::endl;
                uint16_t randomOutputPort = tunSmolStack.randomOutputPort();
                tunSmolStack.connectIpv4(socketHandle, CIpv4Address{
                                             {172,217,28,238}},
                                         randomOutputPort, 80);
                state = State::Request;
            }
            if (state == State::Request)
            {
                std::string httpRequestData("GET /index.html HTTP/1.1\r\n\
                    Host: www.google.com\r\n\
                    Connection: Keep-Alive\r\n\
                    \r\n");
                std::cout << "HTTP: " << httpRequestData << std::endl;
                const uint8_t* httpRequestDataBuffer = reinterpret_cast<const uint8_t*>(httpRequestData.c_str());
                tunSmolStack.send(socketHandle, httpRequestDataBuffer, httpRequestData.size(), endpointNone);
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
