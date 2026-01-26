set(LIBS
    MLIRBril
    LLVMSupport
  )
add_llvm_executable(bril2mlir bril2mlir.cpp)

llvm_update_compile_flags(bril2mlir)
target_link_libraries(bril2mlir PRIVATE ${LIBS})

mlir_check_all_link_libraries(bril2mlir)
