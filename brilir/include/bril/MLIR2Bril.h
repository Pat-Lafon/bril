//===- MLIR2Bril.h - Convert MLIR to Bril JSON
//------------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//
//
// This file declares a simple interface to convert MLIR Bril dialect
// to a JSON representation for the Bril language.
//
//===----------------------------------------------------------------------===//

#ifndef BRIL_MLIR2BRIL_H
#define BRIL_MLIR2BRIL_H

#include "mlir/IR/BuiltinOps.h"
#include "llvm/Support/JSON.h"

namespace bril {
llvm::json::Value mlirToBril(mlir::ModuleOp module);
} // namespace bril

#endif // BRIL_MLIR2BRIL_H
