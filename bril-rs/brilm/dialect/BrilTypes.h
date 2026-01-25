//===- BrilTypes.h - Bril dialect types -----------------------*- C++ -*-===//

#ifndef BRIL_TYPES_H
#define BRIL_TYPES_H

#include "mlir/IR/BuiltinTypes.h"
#include "mlir/IR/DialectImplementation.h"
#include "llvm/ADT/TypeSwitch.h"

#include "BrilDialect.h"

#define GET_TYPEDEF_CLASSES
#include "bril/BrilTypesTypes.h.inc"

#endif // BRIL_TYPES_H
