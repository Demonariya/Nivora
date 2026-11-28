cmake_minimum_required(VERSION 3.21)

project(nivora_cef_native_host LANGUAGES CXX)

if(NOT WIN32 OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
  message(FATAL_ERROR "Nivora CEF Native Host currently supports Windows x64 only.")
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

list(APPEND CMAKE_MODULE_PATH "${CEF_ROOT}/cmake")
find_package(CEF REQUIRED)

add_subdirectory("${CEF_LIBCEF_DLL_WRAPPER_PATH}" libcef_dll_wrapper)

set(CEF_TARGET "nivora-cef-host")
set(CEF_TARGET_OUT_DIR "${CMAKE_CURRENT_BINARY_DIR}/out")
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${CEF_TARGET_OUT_DIR}")

foreach(config DEBUG RELEASE RELWITHDEBINFO MINSIZEREL)
  set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_${config} "${CEF_TARGET_OUT_DIR}")
endforeach()

ADD_LOGICAL_TARGET("libcef_lib" "${CEF_LIB_DEBUG}" "${CEF_LIB_RELEASE}")

add_executable(${CEF_TARGET} WIN32 src/main.cpp)
SET_EXECUTABLE_TARGET_PROPERTIES(${CEF_TARGET})
add_dependencies(${CEF_TARGET} libcef_dll_wrapper)

target_include_directories(${CEF_TARGET} PRIVATE "${CEF_INCLUDE_PATH}")
target_link_libraries(${CEF_TARGET}
  PRIVATE
    libcef_lib
    libcef_dll_wrapper
    ${CEF_STANDARD_LIBS}
)

if(USE_SANDBOX)
  ADD_LOGICAL_TARGET("cef_sandbox_lib"
    "${CEF_SANDBOX_LIB_DEBUG}"
    "${CEF_SANDBOX_LIB_RELEASE}"
  )
  target_link_libraries(${CEF_TARGET}
    PRIVATE
      cef_sandbox_lib
      ${CEF_SANDBOX_STANDARD_LIBS}
  )
else()
  message(FATAL_ERROR "Nivora's native smoke host requires the CEF Windows sandbox.")
endif()

if(COMMAND ADD_WINDOWS_MANIFEST)
  ADD_WINDOWS_MANIFEST("${CMAKE_CURRENT_SOURCE_DIR}" "${CEF_TARGET}" "exe")
endif()

COPY_FILES("${CEF_TARGET}" "${CEF_BINARY_FILES}" "${CEF_BINARY_DIR}" "${CEF_TARGET_OUT_DIR}")
COPY_FILES("${CEF_TARGET}" "${CEF_RESOURCE_FILES}" "${CEF_RESOURCE_DIR}" "${CEF_TARGET_OUT_DIR}")

message(STATUS "Nivora CEF Native Host configured against: ${CEF_ROOT}")
message(STATUS "CEF version: ${CEF_VERSION}")
