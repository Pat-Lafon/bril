use std::sync::OnceLock;

use mmtk::vm::VMBinding;
use mmtk::MMTK;

pub mod active_plan;
pub mod api;
pub mod collection;
pub mod object_model;
pub mod reference_glue;
pub mod scanning;

pub type DummyVMSlot = mmtk::vm::slot::SimpleSlot;

#[derive(Default)]
pub struct BrilGC;

#[derive(Debug, Clone)]
pub struct MMTKBrilPointer;

impl crate::allocator::BrilPointer for MMTKBrilPointer {
  fn add(&self, offset: i64) -> Self {
    todo!()
  }
}

impl From<&crate::allocator::Value<Self>> for MMTKBrilPointer {
  fn from(value: &crate::allocator::Value<Self>) -> Self {
    if let crate::allocator::Value::Pointer(p) = value {
      p.clone()
    } else {
      unreachable!()
    }
  }
}

// Documentation: https://docs.mmtk.io/api/mmtk/vm/trait.VMBinding.html
impl VMBinding for BrilGC {
  type VMObjectModel = object_model::VMObjectModel;
  type VMScanning = scanning::VMScanning;
  type VMCollection = collection::VMCollection;
  type VMActivePlan = active_plan::VMActivePlan;
  type VMReferenceGlue = reference_glue::VMReferenceGlue;
  type VMSlot = DummyVMSlot;
  type VMMemorySlice = mmtk::vm::slot::UnimplementedMemorySlice;

  /// Allowed maximum alignment in bytes.
  const MAX_ALIGNMENT: usize = 1 << 6;
}

use mmtk::util::{Address, ObjectReference};

impl BrilGC {
  pub fn object_start_to_ref(start: Address) -> ObjectReference {
    // Safety: start is the allocation result, and it should not be zero with an offset.
    unsafe {
      ObjectReference::from_raw_address_unchecked(
        start + crate::mmtk_bril::object_model::OBJECT_REF_OFFSET,
      )
    }
  }
}

pub static SINGLETON: OnceLock<Box<MMTK<BrilGC>>> = OnceLock::new();

fn mmtk() -> &'static MMTK<BrilGC> {
  SINGLETON.get().unwrap()
}
