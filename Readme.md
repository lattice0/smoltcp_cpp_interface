# SmolTCP C and C++ interface

This is a project aimed towards making the great Rust SmolTCP library acessible through C++ (and incidentally through C).

I created a simple C interface and I compile and link with SmolTCP's library. Support for TUN, TAP and VirtualTun are almost ready.

The idea of this project is to integrate with OpenVPN3's C++ library in order to use it as a true library, without relying on the system's TUN/TAP interface.