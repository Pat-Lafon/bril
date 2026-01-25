//===- BrilOpsImpl.cpp - Bril dialect op implementations ----*- C++ -*-===//
//
// Custom verifier implementations for Bril dialect operations.
// Used by melior-build to provide type checking for LoadOp and StoreOp.
//
//===----------------------------------------------------------------------===//

#include "BrilOps.h"
#include "mlir/Support/LogicalResult.h"

using namespace mlir;
using namespace mlir::bril;

// Include generated type implementations
#define GET_TYPEDEF_CLASSES
#include "bril/BrilTypesTypes.cpp.inc"

// Include generated operation implementations
#define GET_OP_CLASSES
#include "bril/BrilOps.cpp.inc"

//===----------------------------------------------------------------------===//
// LoadOp verifier
//===----------------------------------------------------------------------===//

llvm::LogicalResult LoadOp::verify() {
  auto ptrType = llvm::dyn_cast<PtrType>(getPtr().getType());
  if (!ptrType)
    return emitOpError("expected 'bril.ptr' type for 'ptr' operand");

  if (getResult().getType() != ptrType.getPointeeType())
    return emitOpError("result type must match pointee type of pointer");

  return success();
}

//===----------------------------------------------------------------------===//
// StoreOp verifier
//===----------------------------------------------------------------------===//

llvm::LogicalResult StoreOp::verify() {
  auto ptrType = llvm::dyn_cast<PtrType>(getPtr().getType());
  if (!ptrType)
    return emitOpError("expected 'bril.ptr' type for 'ptr' operand");

  if (getValue().getType() != ptrType.getPointeeType())
    return emitOpError("value type must match pointee type of pointer");

  return success();
}
