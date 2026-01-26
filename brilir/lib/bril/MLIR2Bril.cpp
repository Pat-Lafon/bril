#include "bril/BrilOps.h"
#include "bril/BrilTypes.h"
#include "bril/MLIR2Bril.h"
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
#include "llvm/Support/Casting.h"
#include "llvm/Support/JSON.h"
#include "llvm/Support/LogicalResult.h"
#include "llvm/Support/raw_ostream.h"
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <string>
#include <unordered_map>
#include <vector>

using namespace mlir::bril;
using namespace bril;

using llvm::dyn_cast;
using llvm::isa;
using llvm::SmallVector;
using llvm::StringRef;

namespace {
class MLIR2BrilImpl {
public:
  MLIR2BrilImpl() { DEBUG = getenv("DEBUG") != nullptr; }

  llvm::json::Value brilGenModule(mlir::ModuleOp module) {
    if (DEBUG)
      llvm::errs() << "entering function mlirGenModule\n";

    llvm::json::Object brilJson;
    llvm::json::Array functions;

    for (auto fn : module.getOps<mlir::bril::FuncOp>()) {
      auto funcJson = brilGenFunc(fn);
      functions.push_back(std::move(funcJson));
    }

    brilJson["functions"] = std::move(functions);

    return llvm::json::Value(std::move(brilJson));
  }

private:
  bool DEBUG;
  llvm::DenseMap<mlir::Value, std::string> idMap;
  llvm::DenseMap<mlir::Block *, std::string> blockLabels;

  llvm::json::Value getTypeJson(mlir::Type type) {
    if (type.isInteger(64))
      return "int";
    if (type.isInteger(1))
      return "bool";
    if (auto ptrType = dyn_cast<mlir::bril::PtrType>(type)) {
      if (ptrType.getPointeeType().isInteger(64)) {
        llvm::json::Object obj;
        obj["ptr"] = "int";
        return llvm::json::Value(std::move(obj));
      } else if (ptrType.getPointeeType().isInteger(1)) {
        llvm::json::Object obj;
        obj["ptr"] = "bool";
        return llvm::json::Value(std::move(obj));
      } else {
        llvm::errs() << "Unsupported pointee type in getTypeJson: "
                     << ptrType.getPointeeType() << "\n";
        abort();
      }
    }
    llvm::errs() << "Unsupported type in getTypeString: " << type << "\n";
    abort();
  }

  std::string getId(mlir::Value v) {
    auto it = idMap.find(v);
    if (it != idMap.end())
      return it->second;
    idMap[v] = "v" + std::to_string(idMap.size());
    return idMap[v];
  }

  std::string getBlockLabel(mlir::Block *block) {
    auto it = blockLabels.find(block);
    if (it != blockLabels.end())
      return it->second;
    blockLabels[block] = "bb" + std::to_string(blockLabels.size());
    return blockLabels[block];
  }

  llvm::json::Value brilGenOp(mlir::Operation &op) {
    if (DEBUG)
      llvm::errs() << "entering function brilGenOp\n";

    if (auto constOp = dyn_cast<ConstantOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "const";
      instrJson["dest"] = getId(constOp.getResult());
      if (auto intAttr = dyn_cast<mlir::IntegerAttr>(constOp.getValue())) {
        if (intAttr.getType().isInteger(64)) {
          instrJson["type"] = "int";
          instrJson["value"] = intAttr.getInt();
        } else if (intAttr.getType().isInteger(1)) {
          instrJson["type"] = "bool";
          instrJson["value"] = static_cast<bool>(intAttr.getInt());
        } else {
          llvm::errs() << "Unsupported constant type in brilGenOp: "
                       << intAttr.getType() << "\n";
          abort();
        }
      }
      return llvm::json::Value(std::move(instrJson));
    } else if (auto undefOp = dyn_cast<UndefOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "undef";
      instrJson["dest"] = getId(undefOp.getResult());
      instrJson["type"] = getTypeJson(undefOp.getResult().getType());
      return llvm::json::Value(std::move(instrJson));
    } else if (isa<AddOp>(op) || isa<SubOp>(op) || isa<MulOp>(op) ||
               isa<DivOp>(op) || isa<EqOp>(op) || isa<LtOp>(op) ||
               isa<GtOp>(op) || isa<LeOp>(op) || isa<GeOp>(op) ||
               isa<AndOp>(op) || isa<OrOp>(op)) {
      llvm::json::Object instrJson;
      if (isa<AddOp>(op))
        instrJson["op"] = "add";
      else if (isa<SubOp>(op))
        instrJson["op"] = "sub";
      else if (isa<MulOp>(op))
        instrJson["op"] = "mul";
      else if (isa<DivOp>(op))
        instrJson["op"] = "div";
      else if (isa<EqOp>(op))
        instrJson["op"] = "eq";
      else if (isa<LtOp>(op))
        instrJson["op"] = "lt";
      else if (isa<GtOp>(op))
        instrJson["op"] = "gt";
      else if (isa<LeOp>(op))
        instrJson["op"] = "le";
      else if (isa<GeOp>(op))
        instrJson["op"] = "ge";
      else if (isa<AndOp>(op))
        instrJson["op"] = "and";
      else if (isa<OrOp>(op))
        instrJson["op"] = "or";

      instrJson["dest"] = getId(op.getResult(0));
      llvm::json::Array args;
      for (auto operand : op.getOperands()) {
        args.push_back(getId(operand));
      }
      instrJson["args"] = std::move(args);
      instrJson["type"] = getTypeJson(op.getResult(0).getType());

      return llvm::json::Value(std::move(instrJson));
    } else if (auto idOp = dyn_cast<IdOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "id";
      instrJson["dest"] = getId(idOp.getResult());
      llvm::json::Array args;
      args.push_back(getId(idOp.getInput()));
      instrJson["args"] = std::move(args);
      instrJson["type"] = getTypeJson(idOp.getResult().getType());
      return llvm::json::Value(std::move(instrJson));
    } else if (auto notOp = dyn_cast<NotOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "not";
      instrJson["dest"] = getId(notOp.getResult());
      llvm::json::Array args;
      args.push_back(getId(notOp->getOperand(0)));
      instrJson["args"] = std::move(args);
      instrJson["type"] = getTypeJson(notOp.getResult().getType());
      return llvm::json::Value(std::move(instrJson));
    } else if (auto callOp = dyn_cast<CallOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "call";
      llvm::json::Array funcs;
      funcs.push_back(callOp.getCallee().str());
      instrJson["funcs"] = std::move(funcs);
      llvm::json::Array args;
      for (auto operand : callOp.getInputs()) {
        args.push_back(getId(operand));
      }
      instrJson["args"] = std::move(args);
      if (!callOp.getResults().empty()) {
        instrJson["dest"] = getId(callOp.getResult(0));
        instrJson["type"] = getTypeJson(callOp.getResult(0).getType());
      }
      return llvm::json::Value(std::move(instrJson));
    } else if (auto brOp = dyn_cast<BrOp>(op)) {
      llvm::json::Array instrArray;

      auto trueBlock = brOp.getTrueTarget();
      auto falseBlock = brOp.getFalseTarget();

      for (auto entry :
           llvm::zip(trueBlock->getArguments(), brOp.getTrueArgs())) {
        auto arg = std::get<0>(entry);
        auto value = std::get<1>(entry);
        // insert set operation
        llvm::json::Object setInstrJson;
        setInstrJson["op"] = "set";
        llvm::json::Array setArgs;
        setArgs.push_back(getId(arg));
        setArgs.push_back(getId(value));
        setInstrJson["args"] = std::move(setArgs);

        instrArray.push_back(llvm::json::Value(std::move(setInstrJson)));
      }

      for (auto entry :
           llvm::zip(falseBlock->getArguments(), brOp.getFalseArgs())) {
        auto arg = std::get<0>(entry);
        auto value = std::get<1>(entry);
        // insert set operation
        llvm::json::Object setInstrJson;
        setInstrJson["op"] = "set";
        llvm::json::Array setArgs;
        setArgs.push_back(getId(arg));
        setArgs.push_back(getId(value));
        setInstrJson["args"] = std::move(setArgs);

        instrArray.push_back(llvm::json::Value(std::move(setInstrJson)));
      }

      llvm::json::Object brInstrJson;
      brInstrJson["op"] = "br";
      llvm::json::Array brArgs;
      brArgs.push_back(getId(brOp.getCondition()));
      brInstrJson["args"] = std::move(brArgs);
      llvm::json::Array labels;
      labels.push_back(getBlockLabel(brOp.getTrueTarget()));
      labels.push_back(getBlockLabel(brOp.getFalseTarget()));
      brInstrJson["labels"] = std::move(labels);

      instrArray.push_back(llvm::json::Value(std::move(brInstrJson)));

      return llvm::json::Value(std::move(instrArray));
    } else if (auto jmpOp = dyn_cast<JmpOp>(op)) {
      llvm::json::Array instrArray;

      for (auto entry :
           llvm::zip(jmpOp.getTarget()->getArguments(), jmpOp.getArgs())) {
        auto arg = std::get<0>(entry);
        auto value = std::get<1>(entry);
        // insert set operation
        llvm::json::Object setInstrJson;
        setInstrJson["op"] = "set";
        llvm::json::Array setArgs;
        setArgs.push_back(getId(arg));
        setArgs.push_back(getId(value));
        setInstrJson["args"] = std::move(setArgs);

        instrArray.push_back(llvm::json::Value(std::move(setInstrJson)));
      }

      llvm::json::Object jmpInstrJson;
      jmpInstrJson["op"] = "jmp";
      llvm::json::Array labels;
      labels.push_back(getBlockLabel(jmpOp.getTarget()));
      jmpInstrJson["labels"] = std::move(labels);

      instrArray.push_back(llvm::json::Value(std::move(jmpInstrJson)));
      return llvm::json::Value(std::move(instrArray));
    } else if (auto retOp = dyn_cast<RetOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "ret";
      if (retOp.getReturnValue()) {
        llvm::json::Array args;
        args.push_back(getId(retOp.getReturnValue()));
        instrJson["args"] = std::move(args);
      }
      return llvm::json::Value(std::move(instrJson));
    } else if (auto printOp = dyn_cast<PrintOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "print";
      llvm::json::Array args;
      for (auto operand : printOp.getValues()) {
        args.push_back(getId(operand));
      }
      instrJson["args"] = std::move(args);
      return llvm::json::Value(std::move(instrJson));
    } else if (auto nopOp = dyn_cast<NopOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "nop";
      return llvm::json::Value(std::move(instrJson));
    } else if (auto allocOp = dyn_cast<AllocOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "alloc";
      instrJson["dest"] = getId(allocOp.getResult());
      instrJson["type"] = getTypeJson(allocOp.getResult().getType());
      llvm::json::Array args;
      args.push_back(getId(allocOp.getSize()));
      instrJson["args"] = std::move(args);
      return llvm::json::Value(std::move(instrJson));
    } else if (auto freeOp = dyn_cast<FreeOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "free";
      llvm::json::Array args;
      args.push_back(getId(freeOp.getPtr()));
      instrJson["args"] = std::move(args);
      return llvm::json::Value(std::move(instrJson));
    } else if (auto loadOp = dyn_cast<LoadOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "load";
      instrJson["dest"] = getId(loadOp.getResult());
      instrJson["type"] = getTypeJson(loadOp.getResult().getType());
      llvm::json::Array args;
      args.push_back(getId(loadOp.getPtr()));
      instrJson["args"] = std::move(args);
      return llvm::json::Value(std::move(instrJson));
    } else if (auto storeOp = dyn_cast<StoreOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "store";
      llvm::json::Array args;
      args.push_back(getId(storeOp.getPtr()));
      args.push_back(getId(storeOp.getValue()));
      instrJson["args"] = std::move(args);
      return llvm::json::Value(std::move(instrJson));
    } else if (auto ptrAddOp = dyn_cast<PtrAddOp>(op)) {
      llvm::json::Object instrJson;
      instrJson["op"] = "ptradd";
      instrJson["dest"] = getId(ptrAddOp.getResult());
      instrJson["type"] = getTypeJson(ptrAddOp.getResult().getType());
      llvm::json::Array args;
      args.push_back(getId(ptrAddOp.getPtr()));
      args.push_back(getId(ptrAddOp.getOffset()));
      instrJson["args"] = std::move(args);
      return llvm::json::Value(std::move(instrJson));
    }

    else {
      llvm::errs() << "Unsupported operation in brilGenOp: "
                   << op.getName().getStringRef() << "\n";
    }
    abort();
    return llvm::json::Value(nullptr);
  }

  llvm::json::Array brilGenBlock(mlir::Block &block, bool entryBlock = false) {
    if (DEBUG)
      llvm::errs() << "entering function brilGenBlock\n";

    llvm::json::Array blockJson;

    llvm::json::Object labelObj;
    labelObj["label"] = getBlockLabel(&block);
    blockJson.push_back(llvm::json::Value(std::move(labelObj)));

    if (!entryBlock) {
      for (auto arg : block.getArguments()) {
        llvm::json::Object getJson;
        getJson["dest"] = getId(arg);
        getJson["op"] = "get";
        getJson["type"] = getTypeJson(arg.getType());
        blockJson.push_back(llvm::json::Value(std::move(getJson)));
      }
    }

    for (auto &op : block.getOperations()) {
      auto opJson = brilGenOp(op);
      if (auto *arr = opJson.getAsArray()) {
        for (auto &instrJson : *arr) {
          blockJson.push_back(std::move(instrJson));
        }
      } else {
        blockJson.push_back(std::move(opJson));
      }
    }

    return blockJson;
  }

  llvm::json::Value brilGenFunc(mlir::bril::FuncOp func) {
    if (DEBUG)
      llvm::errs() << "entering function brilGenFunc "
                   << func.getSymName().str() << "\n";

    idMap.clear();
    blockLabels.clear();

    llvm::json::Object funcJson;
    funcJson["name"] = func.getSymName().str();

    llvm::json::Array argsArr;
    for (auto arg : func.getArguments()) {
      llvm::json::Object argJson;
      argJson["name"] = getId(arg);
      argJson["type"] = getTypeJson(arg.getType());
      argsArr.push_back(llvm::json::Value(std::move(argJson)));
    }
    funcJson["args"] = std::move(argsArr);

    if (!func.getFunctionType().getResults().empty()) {
      auto retType = func.getFunctionType().getResult(0);
      funcJson["type"] = getTypeJson(retType);
    }

    for (auto &block : func.getBlocks()) {
      // sequentially number the blocks
      getBlockLabel(&block);
    }

    llvm::json::Array instrsArr;
    bool entryBlock = true;
    for (auto &block : func.getBlocks()) {
      auto blockJson = brilGenBlock(block, entryBlock);
      for (auto &instrJson : blockJson) {
        instrsArr.push_back(std::move(instrJson));
      }
      entryBlock = false;
    }
    funcJson["instrs"] = std::move(instrsArr);

    return llvm::json::Value(std::move(funcJson));
  }
};

} // namespace

namespace bril {
llvm::json::Value mlirToBril(mlir::ModuleOp module) {
  return MLIR2BrilImpl().brilGenModule(module);
}
} // namespace bril
