//===- BrilOps.h - Bril dialect operations ----------------------*- C++ -*-===//

#ifndef BRIL_OPS_H
#define BRIL_OPS_H

#include "mlir/IR/BuiltinTypes.h"
#include "mlir/IR/Builders.h"
#include "mlir/IR/OpDefinition.h"
#include "mlir/IR/OpImplementation.h"
#include "mlir/Interfaces/FunctionInterfaces.h"
#include "mlir/Interfaces/CallInterfaces.h"
#include "mlir/Interfaces/ControlFlowInterfaces.h"
#include "mlir/Interfaces/InferTypeOpInterface.h"
#include "mlir/Interfaces/SideEffectInterfaces.h"
#include "mlir/Bytecode/BytecodeOpInterface.h"

#include "BrilDialect.h"
#include "BrilTypes.h"

#define GET_OP_CLASSES
#include "bril/BrilOps.h.inc"

#endif // BRIL_OPS_H
