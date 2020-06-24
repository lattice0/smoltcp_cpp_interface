#ifndef SMOL_TCP_INTERFACE_H
#define SMOL_TCP_INTERFACE_H
#include <unordered_map>
#include <random>
#include <limits>
#include <queue>

typedef void *SmolStackPtr;
typedef size_t SocketHandle;

static const int SOCKET_TCP = 0;
static const int SOCKET_UDP = 1;

template <typename T>
class HandleMap
{
public:
    std::unordered_map<size_t, T> map;

    T &get(size_t key)
    {
        auto it = map.find(key);
        if (it == map.end())
            throw std::invalid_argument("invalid key");
        return it->second;
    }

    size_t emplace(T t)
    {
        size_t handle = getNewHandle();
        map[handle] = t;
        return handle;
    }

private:
    size_t currentHandle = 0;
    size_t getNewHandle()
    {
        if (currentHandle < std::numeric_limits<size_t>::max())
        {
            currentHandle += 1;
            return currentHandle;
        }
        else
        {
            throw std::runtime_error("Reached handle too big, you're using too much sockets\n");
        }
    }
};

struct CIpv4Address
{
    uint8_t address[4];
};

struct CIpv6Address
{
    uint16_t address[8];
};

struct CIpv4Cidr
{
    CIpv4Address address;
    uint32_t prefix;
};

struct CIpv6Cidr
{
    CIpv6Address address;
    uint64_t prefix;
};

enum CIpEndpointType
{
    None = 0,
    Ipv4 = 1,
    Ipv6 = 2
};

struct CIpEndpoint
{
    CIpEndpointType type;
    CIpv4Address ipv4;
    CIpv6Address ipv6;
    uint16_t port;
};

extern "C" void cppDeleteArray(uint8_t *data)
{
    delete[] data;
}

extern "C" void cppDeletePointer(uint8_t *data)
{
    delete data;
}

extern "C" SmolStackPtr smol_stack_smol_stack_new_virtual_tun(const char *interfaceName);
extern "C" SmolStackPtr smol_stack_smol_stack_new_tun(const char *interfaceName);
extern "C" size_t smol_stack_add_socket(SmolStackPtr, uint8_t socketType, SocketHandle socketHandle);
extern "C" void smol_stack_poll(SmolStackPtr);
extern "C" void smol_stack_spin(SmolStackPtr, SocketHandle socketHandle);
extern "C" void smol_stack_tcp_connect_ipv4(SmolStackPtr, SocketHandle socketHandle, CIpv4Address, uint8_t src_port, uint8_t dst_port);
extern "C" void smol_stack_tcp_connect_ipv6(SmolStackPtr, SocketHandle socketHandle, CIpv6Address, uint8_t src_port, uint8_t dst_port);
extern "C" uint8_t smol_stack_smol_socket_send(SmolStackPtr, SocketHandle socketHandle, const uint8_t *data, size_t len, CIpEndpoint endpoint, void *, uint8_t (*)(void *));
extern "C" void smol_stack_add_ipv4_address(SmolStackPtr, CIpv4Cidr);
extern "C" void smol_stack_add_ipv6_address(SmolStackPtr, CIpv6Cidr);
extern "C" void smol_stack_add_default_v4_gateway(SmolStackPtr, CIpv4Address);
extern "C" void smol_stack_add_default_v6_gateway(SmolStackPtr, CIpv6Address);
extern "C" uint8_t smol_stack_finalize(SmolStackPtr);

enum StackType
{
    VirtualTun,
    Tun,
    Tap
};

class RustSlice {
public:
    uint8_t* data;
    size_t len;

    RustSlice(uint8_t* data, size_t len): data(data), len(len) {}

    ~RustSlice() {
        //destroy on Rust
    }
};

class SmolSocket
{
public:
    //Handle is just a number that represents this socket
    //on Rust side.
    SocketHandle handle;
    //TODO: put limit on packets size
    std::queue<RustSlice> packets;
};

template <typename T>
class SmolOwner
{
private:
    /*
        SmolOwner owns the pointer to this type and
        it's responsible for deleting it when it's 
        destructed
    */
    T *t;
    SmolOwner(T *t)
    {
        this->t = t;
    }
    /*
        Both Rust and C++ keep handles to SmolSocket objects. These handles
        are passed from C++ to Rust and from Rust to C++ instead of passing
        pointers to SmolSocket objects, which is unsafe. The values do not need
        to be equal on both sides. 
    */
public:
    //Prevents SmolOwner to be created on stack
    static SmolOwner *allocate(T *t)
    {
        return new SmolOwner(t);
    }

    ~SmolOwner()
    {
        std::cout << "~SmolOwner called" << std::endl;
        delete t;
    }
};

class TunSmolStack
{
private:
    SmolStackPtr smolStackPtr;
    std::random_device rd;
    std::mt19937 mt{rd()};
    std::uniform_int_distribution<int> random{49152, 49152 + 16383};
    size_t currentHandle = 0;
    std::unordered_map<size_t, SmolSocket> smolSocketHandles;

public:
    TunSmolStack(std::string interfaceName, StackType stackType)
    {
        if (stackType == StackType::VirtualTun)
        {
            smolStackPtr = smol_stack_smol_stack_new_virtual_tun(interfaceName.c_str());
        }
        else if (stackType == StackType::Tun)
        {
            std::cout << "creating TUN device" << std::endl;
            smolStackPtr = smol_stack_smol_stack_new_tun(interfaceName.c_str());
        }
        else if (stackType == StackType::Tap)
        {
            //throw error
        }
    }

    size_t getNewHandle()
    {
        if (currentHandle < std::numeric_limits<size_t>::max())
        {
            currentHandle += 1;
            return currentHandle;
        }
        else
        {
            throw std::runtime_error("Reached handle too big, you're using too much sockets\n");
        }
    }

    SmolSocket addSocket(uint8_t socketType)
    {
        size_t handle = getNewHandle();
        uint8_t result = smol_stack_add_socket(smolStackPtr, socketType, handle);
        SmolSocket smolSocket;
        smolSocket.handle = handle;
        smolSocketHandles[handle] = smolSocket;
        return smolSocket;
    }

    /*
    size_t addSmolSocket(SmolSocket smolSocket)
    {
        size_t handle = getNewHandle();
        smolSocketHandles[handle] = smolSocket;
    }
    */

    void poll()
    {
        smol_stack_poll(smolStackPtr);
    }

    void spin(SocketHandle socketHandle)
    {
        smol_stack_spin(smolStackPtr, socketHandle);
    }

    /*
        On the act of send, we specify the handle for the socket, the pointer do the data,
        which is the most important type, and its lenght. For UDP and IGMP sockets we also
        have to pass an endpoint (TCP does not need since we call connect before sending).
        Then, we pass a pointer to `SmolOwner`, which is a class that owns the object that
        owns `uint8_t* data`. We also pass the destructor function, which is the function 
        that accepts the `SmolOwner` pointer and deletes it. This function is supposed to
        be called from Rust when it does not need the data `uint8_t* data` anymore.
    */
    template <typename T>
    void send(size_t handle, const uint8_t *data, size_t len, CIpEndpoint endpoint, SmolOwner<T> *pointerToSmolOwner, uint8_t (*smolOwnerDestructor)(void *))
    {
        smol_stack_smol_socket_send(smolStackPtr, handle, data, len, endpoint, static_cast<void *>(pointerToSmolOwner), smolOwnerDestructor);
    }

    void connectIpv4(size_t socket_handle, CIpv4Address address, uint8_t src_port, uint8_t dst_port)
    {
        smol_stack_tcp_connect_ipv4(smolStackPtr, socket_handle, address, src_port, dst_port);
    }

    uint16_t randomOutputPort()
    {
        return random(mt);
    }

    void connectIpv6(size_t socket_handle, CIpv6Address address, uint8_t src_port, uint8_t dst_port)
    {
        smol_stack_tcp_connect_ipv6(smolStackPtr, socket_handle, address, src_port, dst_port);
    }

    void addIpv4Address(CIpv4Cidr cidr)
    {
        smol_stack_add_ipv4_address(smolStackPtr, cidr);
    }

    void addIpv6Address(CIpv6Cidr cidr)
    {
        smol_stack_add_ipv6_address(smolStackPtr, cidr);
    }

    void addDefaultV4Gateway(CIpv4Address address)
    {
        smol_stack_add_default_v4_gateway(smolStackPtr, address);
    }

    void addDefaultV6Gateway(CIpv6Address address)
    {
        smol_stack_add_default_v6_gateway(smolStackPtr, address);
    }

    uint8_t finalize()
    {
        return smol_stack_finalize(smolStackPtr);
    }

    ~TunSmolStack()
    {
        //smol_stack_add_destroy()
    }
};

#endif //SMOL_TCP_INTERFACE_H