cmake_minimum_required(VERSION 3.00)
project(opengrok_client)

set(CMAKE_CXX_STANDARD 17)

# Epoll

if(MSVC OR MSYS OR MINGW OR WIN32)
    message("-- Using wepoll as epoll")
    set(EPOLL_SRC ${CMAKE_SOURCE_DIR}/src/loop/wepoll.c ${CMAKE_SOURCE_DIR}/src/loop/wepoll.h)

    set(SOCKETS_LIBRARIES wsock32 ws2_32)
else()
    set(EPOLL_SRC "")
    set(SOCKETS_LIBRARIES "")
endif()

# ZSTD linking

if (MSVC OR MSYS OR MINGW OR WIN32)
    set(ZSTD_LINK "${CMAKE_SOURCE_DIR}/win/libzstd.so.1.5.0")

    message("-- Found ZSTD dll at ${ZSTD_LINK}")
else()
    set(ZSTD_LINK "zstd")
endif()

# Loop

add_library(event_loop src/loop/selector.cpp src/loop/selector.hpp
        src/loop/loop.cpp src/loop/loop.hpp ${EPOLL_SRC})

# main

include_directories(src/)
add_library(client_lib src/grokclient/client.cpp src/grokclient/client.hpp
        src/grokclient/proxy_client.cpp src/grokclient/proxy_client.hpp)

add_executable(grokclient main.cpp)

target_link_libraries(client_lib event_loop ${ZSTD_LINK} ${SOCKETS_LIBRARIES})
target_link_libraries(grokclient client_lib event_loop)

