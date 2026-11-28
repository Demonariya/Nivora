cmake_minimum_required(VERSION 3.21)

project(nivora_cef_bootstrap_client LANGUAGES CXX)

if(NOT WIN32 OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
  message(FATAL_ERROR "Nivora CEF bootstrap client currently supports Windows x64 only.")
endif()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT DEFINED CEF_ROOT AND DEFINED ENV{NIVORA_CEF_ROOT})
  file(TO_CMAKE_PATH "$ENV{NIVORA_CEF_ROOT}" CEF_ROOT)
endif()

if(NOT DEFINED CEF_ROOT OR NOT EXISTS "${CEF_ROOT}/cmake/FindCEF.cmake")
  message(FATAL_ERROR
    "Set NIVORA_CEF_ROOT (or -DCEF_ROOT=...) to the extracted CEF Standard Distribution root.")
endif()

if(NOT DEFINED NIVORA_APP_IMPORT_LIB OR NOT EXISTS "${NIVORA_APP_IMPORT_LIB}")
  message(FATAL_ERROR
    "Set NIVORA_APP_IMPORT_LIB to target/release/nivora_app.dll.lib from the Rust cdylib build.")
endif()

list(APPEND CMAKE_MODULE_PATH "${CEF_ROOT}/cmake")
find_package(CEF REQUIRED)
add_subdirectory("${CEF_LIBCEF_DLL_WRAPPER_PATH}" libcef_dll_wrapper)

set(CEF_TARGET "nivora-cef-host")
SET_CEF_TARGET_OUT_DIR()
ADD_LOGICAL_TARGET("libcef_lib" "${CEF_LIB_DEBUG}" "${CEF_LIB_RELEASE}")

add_library(${CEF_TARGET} SHARED src/main.cpp)
target_include_directories(${CEF_TARGET} PRIVATE "${CEF_INCLUDE_PATH}")
target_compile_features(${CEF_TARGET} PRIVATE cxx_std_20)
target_compile_definitions(${CEF_TARGET} PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(${CEF_TARGET}
  PRIVATE
    libcef_lib
    libcef_dll_wrapper
    "${NIVORA_APP_IMPORT_LIB}"
    ${CEF_STANDARD_LIBS}
)

set_target_properties(${CEF_TARGET} PROPERTIES
  OUTPUT_NAME "nivora-cef-host"
  RUNTIME_OUTPUT_DIRECTORY "${CEF_TARGET_OUT_DIR}"
  LIBRARY_OUTPUT_DIRECTORY "${CEF_TARGET_OUT_DIR}"
  ARCHIVE_OUTPUT_DIRECTORY "${CEF_TARGET_OUT_DIR}"
  RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CEF_TARGET_OUT_DIR}"
  LIBRARY_OUTPUT_DIRECTORY_RELEASE "${CEF_TARGET_OUT_DIR}"
  ARCHIVE_OUTPUT_DIRECTORY_RELEASE "${CEF_TARGET_OUT_DIR}"
)

COPY_FILES("${CEF_TARGET}" "${CEF_BINARY_FILES}" "${CEF_BINARY_DIR}" "${CEF_TARGET_OUT_DIR}")
COPY_FILES("${CEF_TARGET}" "${CEF_RESOURCE_FILES}" "${CEF_RESOURCE_DIR}" "${CEF_TARGET_OUT_DIR}")

message(STATUS "Nivora CEF bootstrap client configured against: ${CEF_ROOT}")
message(STATUS "Rust application import library: ${NIVORA_APP_IMPORT_LIB}")
message(STATUS "CEF version: ${CEF_VERSION}")
