cmake_minimum_required(VERSION 3.24)
project(nivora_webkit_bridge LANGUAGES CXX)

if(NOT WIN32)
    message(FATAL_ERROR "The Nivora WebKit bridge is supported only on Windows WinCairo builds.")
endif()
if(NOT DEFINED WEBKIT_INCLUDE_DIR)
    message(FATAL_ERROR "WEBKIT_INCLUDE_DIR must point to the directory containing WebKit/WKBase.h")
endif()

add_library(nivora_webkit_bridge STATIC webkit_bridge.cpp)
target_compile_features(nivora_webkit_bridge PRIVATE cxx_std_17)
target_include_directories(nivora_webkit_bridge PRIVATE "${WEBKIT_INCLUDE_DIR}")
target_compile_definitions(nivora_webkit_bridge PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX)
set_target_properties(nivora_webkit_bridge PROPERTIES
    POSITION_INDEPENDENT_CODE ON
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
    ARCHIVE_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}/lib"
    ARCHIVE_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}/lib"
)
install(TARGETS nivora_webkit_bridge ARCHIVE DESTINATION lib)
