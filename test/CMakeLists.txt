cmake_minimum_required(VERSION 3.10)
project(test_main)

# Set the C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Create an executable target from test_main.cpp
add_executable(test_main test_main.cpp)
