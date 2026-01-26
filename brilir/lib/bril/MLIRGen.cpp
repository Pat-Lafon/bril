//===- MLIRGen.cpp - MLIR Generation from a Bril JSON
//----------------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//
//
// This file implements a simple IR generation targeting MLIR from a Bril JSON
// for the Bril language.
//
//===----------------------------------------------------------------------===//

#include "bril/MLIRGen.h"
#include "bril/BrilOps.h"
#include "mlir/IR/Block.h"
#include "mlir/IR/BuiltinAttributes.h"
#include "mlir/IR/Diagnostics.h"
#include "mlir/IR/TypeRange.h"
#include "mlir/IR/Value.h"

#include "mlir/IR/Builders.h"
#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/BuiltinTypes.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/Verifier.h"
#include "mlir/Support/LLVM.h"

#include "llvm/ADT/STLExtras.h"
#include "llvm/ADT/SmallVector.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/ADT/Twine.h"
#include "llvm/Support/JSON.h"
#include "llvm/Support/LogicalResult.h"
#include "llvm/Support/raw_ostream.h"
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <unordered_map>
#include <vector>

using namespace mlir::bril;
using namespace bril;

using llvm::SmallVector;
using llvm::StringRef;

namespace {

/// Implementation of a simple MLIR emission from the Bril JSON.
///
/// This will emit operations that are specific to the Bril language, preserving
/// the semantics of the language and (hopefully) allow to perform accurate
/// analysis and transformation based on these high level semantics.
class MLIRGenImpl {
public:
  MLIRGenImpl(mlir::MLIRContext &context) : builder(&context) {
    DEBUG = getenv("DEBUG") != nullptr;
  }

  mlir::OwningOpRef<mlir::ModuleOp> mlirGen(llvm::json::Value &json) {
    if (DEBUG)
      llvm::errs() << "entering function mlirGen\n";
    // Create the module.
    theModule = mlir::ModuleOp::create(builder.getUnknownLoc());

    auto *jsonObj = json.getAsObject();
    if (!jsonObj) {
      llvm::errs() << "Expected JSON object at top level\n";
      return nullptr;
    }

    auto *functions = jsonObj->getArray("functions");
    if (!functions) {
      llvm::errs() << "Expected 'functions' array\n";
      return nullptr;
    }

    for (auto &funcJson : *functions) {
      labelToBlock.clear();
      blockList.clear();
      blockToLabel.clear();
      symbolTable.clear();
      if (llvm::failed(mlirGenFunction(funcJson))) {
        theModule->emitError("failed to generate function");
        return nullptr;
      }
    }

    if (llvm::failed(mlir::verify(theModule))) {
      theModule->emitError("module verification failed");
      return nullptr;
    }

    return theModule;
  }

private:
  struct BlockInfo {
    mlir::Block *block;
    llvm::SmallVector<std::string, 4> blockArgs;
    llvm::StringMap<mlir::Value> ssaSets;
  };

  mlir::OpBuilder builder;
  mlir::ModuleOp theModule;
  std::unordered_map<std::string, mlir::Value> symbolTable;
  std::unordered_map<std::string, BlockInfo> labelToBlock;
  std::unordered_map<mlir::Block *, std::string> blockToLabel;
  std::vector<mlir::Block *> blockList;
  bool DEBUG;

  llvm::LogicalResult declare(std::string var, mlir::Value value) {
    if (symbolTable.count(var))
      return mlir::failure();
    symbolTable[var] = value;
    return mlir::success();
  }

  mlir::Type getType(const llvm::json::Value &type) {
    if (auto str = type.getAsString()) {
      if (*str == "int")
        return builder.getIntegerType(64);
      if (*str == "bool")
        return builder.getIntegerType(1);
    }
    if (auto *obj = type.getAsObject()) {
      if (auto ptr = obj->getString("ptr")) {
        if (*ptr == "int")
          return mlir::bril::PtrType::get(builder.getContext(),
                                          builder.getIntegerType(64));
        if (*ptr == "bool")
          return mlir::bril::PtrType::get(builder.getContext(),
                                          builder.getIntegerType(1));
      }
    }
    return nullptr;
  }

  std::string generateBlockName() {
    static int blockCounter = 0;
    return "___generated_block_" + std::to_string(blockCounter++);
  }

  std::vector<std::vector<llvm::json::Value>>
  splitBlocks(llvm::json::Array &instrsJson) {
    std::vector<std::vector<llvm::json::Value>> blocks = {};
    std::vector<llvm::json::Value> currentBlock = {};

    for (auto &instrJson : instrsJson) {
      auto *instrObj = instrJson.getAsObject();
      if (instrObj && instrObj->get("op")) {
        currentBlock.push_back(std::move(instrJson));

        auto op = instrObj->getString("op");

        if (op && (*op == "br" || *op == "jmp" || *op == "ret")) {
          blocks.push_back(std::move(currentBlock));
          currentBlock = {};
        }
      } else {
        if (!currentBlock.empty()) {
          blocks.push_back(std::move(currentBlock));
        }

        currentBlock = {std::move(instrJson)};
      }
    }

    if (!currentBlock.empty()) {
      blocks.push_back(std::move(currentBlock));
    }

    return blocks;
  }

  llvm::LogicalResult mlirGenFunction(llvm::json::Value &funcJsonVal) {
    auto *funcJson = funcJsonVal.getAsObject();
    if (!funcJson) {
      llvm::errs() << "Expected function to be an object\n";
      return llvm::failure();
    }

    auto funcNameOpt = funcJson->getString("name");
    if (!funcNameOpt) {
      llvm::errs() << "Function missing 'name'\n";
      return llvm::failure();
    }
    std::string funcName = funcNameOpt->str();

    if (DEBUG)
      llvm::errs() << "entering function mlirGenFunction " << funcName << "\n";

    mlir::SmallVector<mlir::Type, 4> argTypes = {};
    if (auto *args = funcJson->getArray("args")) {
      for (auto &arg : *args) {
        auto *argObj = arg.getAsObject();
        if (!argObj)
          continue;
        auto *typeVal = argObj->get("type");
        if (!typeVal)
          continue;
        auto argType = getType(*typeVal);
        argTypes.push_back(argType);
      }
    }

    mlir::TypeRange returnTypes = {};
    if (auto *typeVal = funcJson->get("type")) {
      returnTypes = {getType(*typeVal)};
    }

    builder.setInsertionPointToEnd(theModule.getBody());

    auto func = FuncOp::create(builder, builder.getUnknownLoc(), funcName,
                               builder.getFunctionType(argTypes, returnTypes));

    auto &entryBlock = func.front();
    if (auto *args = funcJson->getArray("args")) {
      size_t idx = 0;
      for (auto &arg : *args) {
        auto *argObj = arg.getAsObject();
        if (!argObj)
          continue;
        auto nameOpt = argObj->getString("name");
        if (!nameOpt)
          continue;
        std::string name = nameOpt->str();
        auto value = entryBlock.getArgument(idx++);

        if (llvm::failed(declare(name, value))) {
          func.emitError("failed to declare argument ") << name;
          return llvm::failure();
        }
      }
    }

    builder.setInsertionPointToEnd(&entryBlock);

    auto *instrsArr = funcJson->getArray("instrs");
    if (!instrsArr) {
      llvm::errs() << "Function missing 'instrs'\n";
      return llvm::failure();
    }

    auto blocks = splitBlocks(*instrsArr);

    bool firstBlock = true;
    for (auto &block : blocks) {
      if (block.empty())
        continue;

      auto *firstInstr = block.front().getAsObject();
      bool hasLabel = firstInstr && firstInstr->get("label");

      std::string labelName;
      if (hasLabel) {
        labelName = firstInstr->getString("label")->str();
      } else {
        labelName = generateBlockName();
      }

      auto *mlirBlock = firstBlock ? &entryBlock : func.addBlock();
      llvm::SmallVector<std::string, 4> blockArgNames;

      for (auto &instr : block) {
        auto *instrObj = instr.getAsObject();
        if (!instrObj)
          continue;
        // collect all block arguments from 'get' instructions
        auto opOpt = instrObj->getString("op");
        if (opOpt && *opOpt == "get") {
          auto *typeVal = instrObj->get("type");
          if (!typeVal)
            continue;
          auto blockArg =
              mlirBlock->addArgument(getType(*typeVal), builder.getUnknownLoc());
          auto destOpt = instrObj->getString("dest");
          if (!destOpt)
            continue;
          std::string destName = destOpt->str();
          blockArgNames.push_back(destName);
          if (llvm::failed(declare(destName, blockArg))) {
            func.emitError("failed to declare block argument ");
            return llvm::failure();
          }
        }
      }

      labelToBlock[labelName] = BlockInfo{mlirBlock, blockArgNames, {}};
      blockToLabel[mlirBlock] = labelName;
      blockList.push_back(mlirBlock);

      // If block didn't have a label, add one to the first instruction
      if (!hasLabel && !block.empty()) {
        // We'll handle this by adding a label field when processing
      }

      firstBlock = false;
    }

    size_t blockIdx = 0;
    for (auto &block : blocks) {
      if (block.empty()) {
        blockIdx++;
        continue;
      }

      BlockInfo *blockInfo = nullptr;
      auto *firstInstr = block.front().getAsObject();
      std::string labelName;

      if (firstInstr && firstInstr->get("label")) {
        labelName = firstInstr->getString("label")->str();
      } else {
        labelName = blockToLabel[blockList[blockIdx]];
      }

      if (!labelToBlock.count(labelName)) {
        llvm::errs() << "Undefined label: " << labelName << "\n";
        return llvm::failure();
      }
      blockInfo = &labelToBlock[labelName];

      builder.setInsertionPointToEnd(blockList[blockIdx]);
      for (auto &instr : block) {
        if (llvm::failed(mlirGenInstruction(instr, blockInfo))) {
          func.emitError("failed to generate instruction");
          return llvm::failure();
        }
      }

      // Check if block needs a terminator
      bool needsTerminator = true;
      if (!block.empty()) {
        auto *lastInstr = block.back().getAsObject();
        if (lastInstr) {
          auto opOpt = lastInstr->getString("op");
          if (opOpt && (*opOpt == "br" || *opOpt == "jmp" || *opOpt == "ret")) {
            needsTerminator = false;
          }
        }
      }

      if (needsTerminator) {
        // create jmp to the next block if it exists
        if (blockIdx + 1 < blockList.size()) {
          std::string targetLabel = blockToLabel[blockList[blockIdx + 1]];
          auto &targetBlock = labelToBlock[targetLabel];

          llvm::SmallVector<mlir::Value, 4> args = {};
          if (blockInfo) {
            for (auto &arg : targetBlock.blockArgs) {
              if (!blockInfo->ssaSets.count(arg)) {
                llvm::errs() << "Undefined variable in jmp args: " << arg
                             << "\n";
                return llvm::failure();
              }
              args.push_back(blockInfo->ssaSets.lookup(arg));
            }
          }
          JmpOp::create(builder, builder.getUnknownLoc(), args,
                        targetBlock.block);
        } else {
          // otherwise just generate a dummy ret
          auto *typeVal = funcJson->get("type");
          if (!typeVal || (typeVal->getAsObject() &&
                           typeVal->getAsObject()->get("ptr"))) {
            RetOp::create(builder, builder.getUnknownLoc(), mlir::Value{});
          } else {
            UndefOp undefOp = UndefOp::create(builder, builder.getUnknownLoc(),
                                              getType(*typeVal));
            RetOp::create(builder, builder.getUnknownLoc(),
                          undefOp.getResult());
          }
        }
      }

      blockIdx++;
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenInstruction(llvm::json::Value &instrJsonVal,
                                         BlockInfo *blockInfo) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (!instrJson) {
      return llvm::success(); // Skip non-objects
    }

    if (DEBUG)
      llvm::errs() << "entering function mlirGenInstruction "
                   << llvm::formatv("{0}", instrJsonVal) << " " << blockInfo
                   << "\n";

    if (!instrJson->get("op")) {
      // label instruction, already handled in block generation
      return llvm::success();
    }

    auto opOpt = instrJson->getString("op");
    if (!opOpt) {
      return llvm::success();
    }
    std::string op = opOpt->str();

    if (op == "get") {
      // already handled in block generation
      return llvm::success();
    } else if (op == "const") {
      return mlirGenConst(instrJsonVal);
    } else if (op == "add" || op == "sub" || op == "mul" || op == "div" ||
               op == "eq" || op == "lt" || op == "gt" || op == "le" ||
               op == "ge" || op == "and" || op == "or") {
      return mlirGenBinaryOp(instrJsonVal);
    } else if (op == "undef") {
      return mlirGenUndef(instrJsonVal);
    } else if (op == "id") {
      return mlirGenId(instrJsonVal);
    } else if (op == "not") {
      return mlirGenNot(instrJsonVal);
    } else if (op == "br") {
      return mlirGenBranch(instrJsonVal, blockInfo);
    } else if (op == "jmp") {
      return mlirGenJmp(instrJsonVal, blockInfo);
    } else if (op == "ret") {
      return mlirGenRet(instrJsonVal);
    } else if (op == "set") {
      return mlirGenSet(instrJsonVal, blockInfo);
    } else if (op == "print") {
      return mlirGenPrint(instrJsonVal);
    } else if (op == "nop") {
      return mlirGenNop(instrJsonVal);
    } else if (op == "call") {
      return mlirGenCall(instrJsonVal);
    } else if (op == "alloc") {
      return mlirGenAlloc(instrJsonVal);
    } else if (op == "load") {
      return mlirGenLoad(instrJsonVal);
    } else if (op == "store") {
      return mlirGenStore(instrJsonVal);
    } else if (op == "free") {
      return mlirGenFree(instrJsonVal);
    } else if (op == "ptradd") {
      return mlirGenPtrAdd(instrJsonVal);
    } else {
      llvm::errs() << "Unhandled operation: " << op << "\n";
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenConst(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenConst "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto destOpt = instrJson->getString("dest");
    if (!destOpt)
      return mlir::failure();
    std::string dest = destOpt->str();

    auto typeOpt = instrJson->getString("type");
    if (typeOpt && *typeOpt == "int") {
      auto valueOpt = instrJson->getInteger("value");
      if (!valueOpt)
        return mlir::failure();
      int64_t value = *valueOpt;
      auto constOp =
          ConstantOp::create(builder, builder.getUnknownLoc(), value);
      if (llvm::failed(declare(dest, constOp.getResult()))) {
        return mlir::failure();
      }
    } else if (typeOpt && *typeOpt == "bool") {
      auto valueOpt = instrJson->getBoolean("value");
      if (!valueOpt)
        return mlir::failure();
      bool value = *valueOpt;
      auto constOp =
          ConstantOp::create(builder, builder.getUnknownLoc(), value);
      if (llvm::failed(declare(dest, constOp.getResult()))) {
        return mlir::failure();
      }
    } else {
      return mlir::failure();
    }
    return llvm::success();
  }

  llvm::LogicalResult mlirGenUndef(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenUndef "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto destOpt = instrJson->getString("dest");
    if (!destOpt)
      return mlir::failure();
    std::string dest = destOpt->str();

    auto *typeVal = instrJson->get("type");
    if (!typeVal)
      return mlir::failure();

    auto undefOp =
        UndefOp::create(builder, builder.getUnknownLoc(), getType(*typeVal));
    if (llvm::failed(declare(dest, undefOp.getResult()))) {
      return mlir::failure();
    }
    return llvm::success();
  }

  llvm::LogicalResult mlirGenId(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenId "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto destOpt = instrJson->getString("dest");
    if (!destOpt)
      return mlir::failure();
    std::string dest = destOpt->str();

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->empty())
      return mlir::failure();

    auto argNameOpt = (*argsArr)[0].getAsString();
    if (!argNameOpt)
      return mlir::failure();
    std::string argName = argNameOpt->str();

    if (!symbolTable.count(argName)) {
      llvm::errs() << "Undefined variable in id operation: " << argName << "\n";
      return mlir::failure();
    }

    auto arg = symbolTable[argName];
    auto *typeVal = instrJson->get("type");
    if (!typeVal)
      return mlir::failure();

    auto idOp = IdOp::create(builder, builder.getUnknownLoc(),
                             getType(*typeVal), arg);

    if (llvm::failed(declare(dest, idOp.getResult()))) {
      llvm::errs() << "Failed to declare variable: " << dest << "\n";
      return mlir::failure();
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenNot(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenNot "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto destOpt = instrJson->getString("dest");
    if (!destOpt)
      return mlir::failure();
    std::string dest = destOpt->str();

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->empty())
      return mlir::failure();

    auto argNameOpt = (*argsArr)[0].getAsString();
    if (!argNameOpt)
      return mlir::failure();
    std::string argName = argNameOpt->str();

    if (!symbolTable.count(argName)) {
      llvm::errs() << "Undefined variable in not operation: " << argName
                   << "\n";
      return mlir::failure();
    }

    auto arg = symbolTable[argName];

    auto notOp = NotOp::create(builder, builder.getUnknownLoc(), arg);
    auto result = notOp.getResult();

    if (llvm::failed(declare(dest, result))) {
      llvm::errs() << "Failed to declare variable: " << dest << "\n";
      return mlir::failure();
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenBranch(llvm::json::Value &instrJsonVal,
                                    BlockInfo *blockInfo) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenBranch "
                   << llvm::formatv("{0}", instrJsonVal) << " " << blockInfo
                   << "\n";
    if (!blockInfo) {
      llvm::errs() << "Branch operation on a block without BlockInfo\n";
      return mlir::failure();
    }

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->empty())
      return mlir::failure();

    auto argNameOpt = (*argsArr)[0].getAsString();
    if (!argNameOpt)
      return mlir::failure();
    std::string argName = argNameOpt->str();

    if (!symbolTable.count(argName)) {
      llvm::errs() << "Undefined variable in branch operation: " << argName
                   << "\n";
      return mlir::failure();
    }

    auto arg = symbolTable[argName];

    auto *labelsArr = instrJson->getArray("labels");
    if (!labelsArr || labelsArr->size() < 2)
      return mlir::failure();

    auto trueLabelOpt = (*labelsArr)[0].getAsString();
    auto falseLabelOpt = (*labelsArr)[1].getAsString();
    if (!trueLabelOpt || !falseLabelOpt)
      return mlir::failure();

    std::string trueLabel = trueLabelOpt->str();
    std::string falseLabel = falseLabelOpt->str();

    if (!labelToBlock.count(trueLabel) || !labelToBlock.count(falseLabel)) {
      llvm::errs() << "Undefined label in branch operation: " << trueLabel
                   << " or " << falseLabel << "\n";
      return mlir::failure();
    }

    auto &trueBlock = labelToBlock[trueLabel];
    auto &falseBlock = labelToBlock[falseLabel];

    llvm::SmallVector<mlir::Value, 4> trueArgs = {};
    llvm::SmallVector<mlir::Value, 4> falseArgs = {};

    for (auto &argStr : trueBlock.blockArgs) {
      if (!blockInfo->ssaSets.count(argStr)) {
        llvm::errs() << "Undefined variable in branch true args: " << argStr
                     << "\n";
        return mlir::failure();
      }
      trueArgs.push_back(blockInfo->ssaSets.lookup(argStr));
    }

    for (auto &argStr : falseBlock.blockArgs) {
      if (!blockInfo->ssaSets.count(argStr)) {
        llvm::errs() << "Undefined variable in branch false args: " << argStr
                     << "\n";
        return mlir::failure();
      }
      falseArgs.push_back(blockInfo->ssaSets.lookup(argStr));
    }

    BrOp::create(builder, builder.getUnknownLoc(), arg, trueArgs, falseArgs,
                 trueBlock.block, falseBlock.block);

    return llvm::success();
  }

  llvm::LogicalResult mlirGenCall(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenCall "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto *funcsArr = instrJson->getArray("funcs");
    if (!funcsArr || funcsArr->empty())
      return mlir::failure();

    auto funcNameOpt = (*funcsArr)[0].getAsString();
    if (!funcNameOpt)
      return mlir::failure();
    std::string funcName = funcNameOpt->str();

    SmallVector<mlir::Value, 4> mlirArgs = {};
    if (auto *argsArr = instrJson->getArray("args")) {
      for (auto &argNameJson : *argsArr) {
        auto argNameOpt = argNameJson.getAsString();
        if (!argNameOpt)
          continue;
        std::string argName = argNameOpt->str();
        if (!symbolTable.count(argName)) {
          llvm::errs() << "Undefined variable in call operation: " << argName
                       << "\n";
          return mlir::failure();
        }
        mlirArgs.push_back(symbolTable[argName]);
      }
    }

    if (instrJson->get("dest")) {
      auto destOpt = instrJson->getString("dest");
      if (!destOpt)
        return mlir::failure();
      std::string dest = destOpt->str();

      auto *typeVal = instrJson->get("type");
      if (!typeVal)
        return mlir::failure();
      auto type = getType(*typeVal);

      auto callOp = CallOp::create(
          builder, builder.getUnknownLoc(), type,
          mlir::FlatSymbolRefAttr::get(builder.getContext(), funcName),
          mlirArgs, mlir::ArrayAttr(), mlir::ArrayAttr());

      if (llvm::failed(declare(dest, callOp.getResult(0)))) {
        llvm::errs() << "Failed to declare variable: " << dest << "\n";
        return mlir::failure();
      }
    } else {
      CallOp::create(
          builder, builder.getUnknownLoc(), mlir::TypeRange{},
          mlir::FlatSymbolRefAttr::get(builder.getContext(), funcName),
          mlirArgs, mlir::ArrayAttr(), mlir::ArrayAttr());
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenPrint(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenPrint "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    SmallVector<mlir::Value, 4> args = {};

    if (auto *argsArr = instrJson->getArray("args")) {
      for (auto &argNameJson : *argsArr) {
        auto argNameOpt = argNameJson.getAsString();
        if (!argNameOpt)
          continue;
        std::string argName = argNameOpt->str();

        if (!symbolTable.count(argName)) {
          llvm::errs() << "Undefined variable in print operation: " << argName
                       << "\n";
          return mlir::failure();
        }

        args.push_back(symbolTable[argName]);
      }
    }

    PrintOp::create(builder, builder.getUnknownLoc(), args);

    return llvm::success();
  }

  llvm::LogicalResult mlirGenNop(llvm::json::Value &instrJsonVal) {
    if (DEBUG)
      llvm::errs() << "entering function mlirGenNop "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";
    NopOp::create(builder, builder.getUnknownLoc());
    return llvm::success();
  }

  llvm::LogicalResult mlirGenRet(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenRet "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto *argsArr = instrJson->getArray("args");
    if (argsArr && !argsArr->empty()) {
      auto argNameOpt = (*argsArr)[0].getAsString();
      if (!argNameOpt)
        return mlir::failure();
      std::string argName = argNameOpt->str();

      if (!symbolTable.count(argName)) {
        llvm::errs() << "Undefined variable in ret operation: " << argName
                     << "\n";
        return mlir::failure();
      }

      auto arg = symbolTable[argName];

      RetOp::create(builder, builder.getUnknownLoc(), arg);
    } else {
      RetOp::create(builder, builder.getUnknownLoc(), mlir::Value{});
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenJmp(llvm::json::Value &instrJsonVal,
                                 BlockInfo *blockInfo) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenJmp "
                   << llvm::formatv("{0}", instrJsonVal) << " " << blockInfo
                   << "\n";

    auto *labelsArr = instrJson->getArray("labels");
    if (!labelsArr || labelsArr->empty())
      return mlir::failure();

    auto targetLabelOpt = (*labelsArr)[0].getAsString();
    if (!targetLabelOpt)
      return mlir::failure();
    std::string targetLabel = targetLabelOpt->str();

    if (!labelToBlock.count(targetLabel)) {
      llvm::errs() << "Undefined label in jmp operation: " << targetLabel
                   << "\n";
      return mlir::failure();
    }

    auto &targetBlock = labelToBlock[targetLabel];

    llvm::SmallVector<mlir::Value, 4> args = {};
    if (blockInfo) {
      for (auto &arg : targetBlock.blockArgs) {
        if (!blockInfo->ssaSets.count(arg)) {
          llvm::errs() << "Undefined variable in jmp args: " << arg << "\n";
          return mlir::failure();
        }
        args.push_back(blockInfo->ssaSets.lookup(arg));
      }
    }

    JmpOp::create(builder, builder.getUnknownLoc(), args, targetBlock.block);

    return llvm::success();
  }

  llvm::LogicalResult mlirGenBinaryOp(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenBinaryOp "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto destOpt = instrJson->getString("dest");
    if (!destOpt)
      return mlir::failure();
    std::string dest = destOpt->str();

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->size() < 2)
      return mlir::failure();

    auto arg1NameOpt = (*argsArr)[0].getAsString();
    auto arg2NameOpt = (*argsArr)[1].getAsString();
    if (!arg1NameOpt || !arg2NameOpt)
      return mlir::failure();

    std::string arg1Name = arg1NameOpt->str();
    std::string arg2Name = arg2NameOpt->str();

    if (!symbolTable.count(arg1Name) || !symbolTable.count(arg2Name)) {
      llvm::errs() << "Undefined variable in binary operation: " << arg1Name
                   << " or " << arg2Name << "\n";
      return mlir::failure();
    }

    auto arg1 = symbolTable[arg1Name];
    auto arg2 = symbolTable[arg2Name];

    mlir::Value result;

    auto opOpt = instrJson->getString("op");
    if (!opOpt)
      return mlir::failure();
    std::string op = opOpt->str();

    if (op == "add") {
      auto addOp = AddOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = addOp.getResult();
    } else if (op == "sub") {
      auto subOp = SubOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = subOp.getResult();
    } else if (op == "mul") {
      auto mulOp = MulOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = mulOp.getResult();
    } else if (op == "div") {
      auto divOp = DivOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = divOp.getResult();
    } else if (op == "eq") {
      auto eqOp = EqOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = eqOp.getResult();
    } else if (op == "lt") {
      auto ltOp = LtOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = ltOp.getResult();
    } else if (op == "gt") {
      auto gtOp = GtOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = gtOp.getResult();
    } else if (op == "le") {
      auto leOp = LeOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = leOp.getResult();
    } else if (op == "ge") {
      auto geOp = GeOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = geOp.getResult();
    } else if (op == "and") {
      auto andOp = AndOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = andOp.getResult();
    } else if (op == "or") {
      auto orOp = OrOp::create(builder, builder.getUnknownLoc(), arg1, arg2);
      result = orOp.getResult();
    }

    if (llvm::failed(declare(dest, result))) {
      llvm::errs() << "Failed to declare variable: " << dest << "\n";
      return mlir::failure();
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenSet(llvm::json::Value &instrJsonVal,
                                 BlockInfo *blockInfo) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenSet "
                   << llvm::formatv("{0}", instrJsonVal) << " " << blockInfo
                   << "\n";
    if (!blockInfo) {
      llvm::errs() << "Set operation on a block without BlockInfo\n";
      return mlir::failure();
    }

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->size() < 2)
      return mlir::failure();

    auto destOpt = (*argsArr)[0].getAsString();
    auto srcOpt = (*argsArr)[1].getAsString();
    if (!destOpt || !srcOpt)
      return mlir::failure();

    std::string dest = destOpt->str();
    std::string src = srcOpt->str();

    mlir::Value arg;

    if (!symbolTable.count(src)) {
      auto destType = symbolTable[dest].getType();
      std::string typeStr = destType.isInteger(1) ? "bool" : "int";

      llvm::json::Object undefJson;
      undefJson["dest"] = "___undef__" + src;
      undefJson["type"] = typeStr;
      undefJson["op"] = "undef";

      llvm::json::Value undefVal(std::move(undefJson));
      (void)mlirGenUndef(undefVal);
      arg = symbolTable["___undef__" + src];
    } else {
      arg = symbolTable[src];
    }

    blockInfo->ssaSets[dest] = arg;

    return llvm::success();
  }

  llvm::LogicalResult mlirGenAlloc(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenAlloc "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto destOpt = instrJson->getString("dest");
    if (!destOpt)
      return mlir::failure();
    std::string dest = destOpt->str();

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->empty())
      return mlir::failure();

    auto sizeNameOpt = (*argsArr)[0].getAsString();
    if (!sizeNameOpt)
      return mlir::failure();
    std::string sizeName = sizeNameOpt->str();

    if (!symbolTable.count(sizeName)) {
      llvm::errs() << "Undefined variable in alloc operation: " << sizeName
                   << "\n";
      return mlir::failure();
    }
    auto size = symbolTable[sizeName];

    auto *typeVal = instrJson->get("type");
    if (!typeVal)
      return mlir::failure();
    auto type = getType(*typeVal);

    auto allocOp =
        AllocOp::create(builder, builder.getUnknownLoc(), type, size);

    if (llvm::failed(declare(dest, allocOp.getResult()))) {
      llvm::errs() << "Failed to declare variable: " << dest << "\n";
      return mlir::failure();
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenFree(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenFree "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->empty())
      return mlir::failure();

    auto ptrNameOpt = (*argsArr)[0].getAsString();
    if (!ptrNameOpt)
      return mlir::failure();
    std::string ptrName = ptrNameOpt->str();

    if (!symbolTable.count(ptrName)) {
      llvm::errs() << "Undefined variable in free operation: " << ptrName
                   << "\n";
      return mlir::failure();
    }
    auto ptr = symbolTable[ptrName];

    FreeOp::create(builder, builder.getUnknownLoc(), ptr);

    return llvm::success();
  }

  llvm::LogicalResult mlirGenLoad(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenLoad "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto destOpt = instrJson->getString("dest");
    if (!destOpt)
      return mlir::failure();
    std::string dest = destOpt->str();

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->empty())
      return mlir::failure();

    auto ptrNameOpt = (*argsArr)[0].getAsString();
    if (!ptrNameOpt)
      return mlir::failure();
    std::string ptrName = ptrNameOpt->str();

    auto *typeVal = instrJson->get("type");
    if (!typeVal)
      return mlir::failure();
    auto type = getType(*typeVal);

    if (!symbolTable.count(ptrName)) {
      llvm::errs() << "Undefined variable in load operation: " << ptrName
                   << "\n";
      return mlir::failure();
    }
    auto ptr = symbolTable[ptrName];

    auto loadOp = LoadOp::create(builder, builder.getUnknownLoc(), type, ptr);

    if (llvm::failed(declare(dest, loadOp.getResult()))) {
      llvm::errs() << "Failed to declare variable: " << dest << "\n";
      return mlir::failure();
    }

    return llvm::success();
  }

  llvm::LogicalResult mlirGenStore(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenStore "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->size() < 2)
      return mlir::failure();

    auto ptrNameOpt = (*argsArr)[0].getAsString();
    auto valueNameOpt = (*argsArr)[1].getAsString();
    if (!ptrNameOpt || !valueNameOpt)
      return mlir::failure();

    std::string ptrName = ptrNameOpt->str();
    std::string valueName = valueNameOpt->str();

    if (!symbolTable.count(ptrName)) {
      llvm::errs() << "Undefined variable in store operation: " << ptrName
                   << "\n";
      return mlir::failure();
    }
    auto ptr = symbolTable[ptrName];

    if (!symbolTable.count(valueName)) {
      llvm::errs() << "Undefined variable in store operation: " << valueName
                   << "\n";
      return mlir::failure();
    }
    auto value = symbolTable[valueName];

    StoreOp::create(builder, builder.getUnknownLoc(), ptr, value);

    return llvm::success();
  }

  llvm::LogicalResult mlirGenPtrAdd(llvm::json::Value &instrJsonVal) {
    auto *instrJson = instrJsonVal.getAsObject();
    if (DEBUG)
      llvm::errs() << "entering function mlirGenPtrAdd "
                   << llvm::formatv("{0}", instrJsonVal) << "\n";

    auto destOpt = instrJson->getString("dest");
    if (!destOpt)
      return mlir::failure();
    std::string dest = destOpt->str();

    auto *argsArr = instrJson->getArray("args");
    if (!argsArr || argsArr->size() < 2)
      return mlir::failure();

    auto ptrNameOpt = (*argsArr)[0].getAsString();
    auto offsetNameOpt = (*argsArr)[1].getAsString();
    if (!ptrNameOpt || !offsetNameOpt)
      return mlir::failure();

    std::string ptrName = ptrNameOpt->str();
    std::string offsetName = offsetNameOpt->str();

    auto *typeVal = instrJson->get("type");
    if (!typeVal)
      return mlir::failure();
    auto type = getType(*typeVal);

    if (!symbolTable.count(ptrName)) {
      llvm::errs() << "Undefined variable in ptradd operation: " << ptrName
                   << "\n";
      return mlir::failure();
    }
    auto ptr = symbolTable[ptrName];

    if (!symbolTable.count(offsetName)) {
      llvm::errs() << "Undefined variable in ptradd operation: " << offsetName
                   << "\n";
      return mlir::failure();
    }
    auto offset = symbolTable[offsetName];

    auto ptrAddOp =
        PtrAddOp::create(builder, builder.getUnknownLoc(), type, ptr, offset);

    if (llvm::failed(declare(dest, ptrAddOp.getResult()))) {
      llvm::errs() << "Failed to declare variable: " << dest << "\n";
      return mlir::failure();
    }

    return llvm::success();
  }
};

} // namespace

namespace bril {

// The public API for codegen.
mlir::OwningOpRef<mlir::ModuleOp> mlirGen(mlir::MLIRContext &context,
                                          llvm::json::Value &json) {
  return MLIRGenImpl(context).mlirGen(json);
}

} // namespace bril
