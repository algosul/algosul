cmake_minimum_required(VERSION 3.28)
project(annasul)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_C_STANDARD 11)
find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK4 REQUIRED gtk4)

add_executable(test_gtk4 main.cpp)

target_include_directories(test_gtk4 PUBLIC ${GTK4_INCLUDE_DIRS})
target_link_directories(test_gtk4 PUBLIC ${GTK4_LIBRARY_DIRS})
target_link_libraries(test_gtk4 ${GTK4_LIBRARIES})

add_executable(test_print test_print.cpp)
