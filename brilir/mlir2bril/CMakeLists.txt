set(LIBS
    MLIRBril
    MLIRParser
    LLVMSupport
  )
add_llvm_executable(mlir2bril mlir2bril.cpp)

llvm_update_compile_flags(mlir2bril)
target_link_libraries(mlir2bril PRIVATE ${LIBS})

mlir_check_all_link_libraries(mlir2bril)
