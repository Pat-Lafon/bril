use fxhash::FxHashMap;
use mimalloc::MiMalloc;

use crate::allocator::{BrilAllocator, BrilPointer, Value};
use crate::error::InterpError;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// todo: This is basically a copy of the heap implement in brili and we could probably do something smarter. This currently isn't that worth it to optimize because most benchmarks do not use the memory extension nor do they run for very long. You (the reader in the future) may be working with bril programs that you would like to speed up that extensively use the bril memory extension. In that case, it would be worth seeing how to implement Heap without a map based memory. Maybe try to re-implement malloc for a large Vec<Value>?
pub struct BasicHeap {
  memory: FxHashMap<usize, Vec<Value<Pointer>>>,
  base_num_counter: usize,
}

impl Default for BasicHeap {
  fn default() -> Self {
    Self {
      memory: FxHashMap::with_capacity_and_hasher(20, fxhash::FxBuildHasher::default()),
      base_num_counter: 0,
    }
  }
}

impl BrilAllocator<Pointer> for BasicHeap {
  fn is_empty(&self) -> bool {
    self.memory.is_empty()
  }

  fn alloc(&mut self, amount: i64) -> Result<Value<Pointer>, InterpError> {
    let amount: usize = amount
      .try_into()
      .map_err(|_| InterpError::CannotAllocSize(amount))?;
    let base = self.base_num_counter;
    self.base_num_counter += 1;
    self.memory.insert(base, vec![Value::default(); amount]);
    Ok(Value::Pointer(Pointer { base, offset: 0 }))
  }

  fn free(&mut self, key: &Pointer) -> Result<(), InterpError> {
    if self.memory.remove(&key.base).is_some() && key.offset == 0 {
      Ok(())
    } else {
      Err(InterpError::IllegalFree(key.base, key.offset))
    }
  }

  fn write(&mut self, key: &Pointer, val: Value<Pointer>) -> Result<(), InterpError> {
    // Will check that key.offset is >=0
    let offset: usize = key
      .offset
      .try_into()
      .map_err(|_| InterpError::InvalidMemoryAccess(key.base, key.offset))?;
    match self.memory.get_mut(&key.base) {
      Some(vec) if vec.len() > offset => {
        vec[offset] = val;
        Ok(())
      }
      Some(_) | None => Err(InterpError::InvalidMemoryAccess(key.base, key.offset)),
    }
  }

  fn read(&self, key: &Pointer) -> Result<&Value<Pointer>, InterpError> {
    // Will check that key.offset is >=0
    let offset: usize = key
      .offset
      .try_into()
      .map_err(|_| InterpError::InvalidMemoryAccess(key.base, key.offset))?;
    self
      .memory
      .get(&key.base)
      .and_then(|vec| vec.get(offset))
      .ok_or(InterpError::InvalidMemoryAccess(key.base, key.offset))
      .and_then(|val| match val {
        Value::Uninitialized => Err(InterpError::UsingUninitializedMemory),
        _ => Ok(val),
      })
  }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Pointer {
  base: usize,
  offset: i64,
}

impl BrilPointer for Pointer {
  fn add(&self, offset: i64) -> Self {
    Self {
      base: self.base,
      offset: self.offset + offset,
    }
  }
}

impl From<&Value<Self>> for Pointer {
  fn from(value: &Value<Self>) -> Self {
    if let Value::Pointer(p) = value {
      *p
    } else {
      unreachable!()
    }
  }
}
