use std::{ops::Range, sync::LazyLock};

use mmtk::{
  util::{constants::LOG_BYTES_IN_WORD, Address, ObjectReference},
  vm::{
    edge_shape::{Edge, MemorySlice},
    ActivePlan, Collection, ObjectModel, ReferenceGlue, Scanning, VMBinding,
  },
  MMTKBuilder, MMTK,
};

// https://github.com/mmtk/mmtk-openjdk/blob/54a249e877e1cbea147a71aafaafb8583f33843d/mmtk/src/lib.rs#L169-L178
pub static BUILDER: LazyLock<MMTKBuilder> = LazyLock::new(|| MMTKBuilder::new());
pub static SINGLETON: LazyLock<Box<MMTK<MMTKHeap>>> =
  LazyLock::new(|| mmtk::memory_manager::mmtk_init(&BUILDER));

#[derive(Debug, Default)]
pub struct MMTKHeap {}

impl VMBinding for MMTKHeap {
  type VMObjectModel = MMTKObjectModel;

  type VMScanning = MMTKScanning;

  type VMCollection = MMTKCollection;

  type VMActivePlan = MMTKActivePlan;

  type VMReferenceGlue = MMTKReferenceGlue;

  type VMEdge = MMTKEdge;

  type VMMemorySlice = MMTKEdgeRange;
}

pub struct MMTKReferenceGlue {}

/// https://github.com/mmtk/mmtk-openjdk/blob/master/mmtk/src/reference_glue.rs
impl ReferenceGlue<MMTKHeap> for MMTKReferenceGlue {
  type FinalizableType = ObjectReference;

  fn get_referent(object: mmtk::util::ObjectReference) -> mmtk::util::ObjectReference {
    todo!()
  }

  fn set_referent(reff: mmtk::util::ObjectReference, referent: mmtk::util::ObjectReference) {
    todo!()
  }

  fn enqueue_references(
    references: &[mmtk::util::ObjectReference],
    tls: mmtk::util::VMWorkerThread,
  ) {
    todo!()
  }
}

pub struct MMTKScanning {}

/// https://github.com/mmtk/mmtk-openjdk/blob/master/mmtk/src/scanning.rs
impl Scanning<MMTKHeap> for MMTKScanning {
  fn scan_object<EV: mmtk::vm::EdgeVisitor<<MMTKHeap as VMBinding>::VMEdge>>(
    tls: mmtk::util::VMWorkerThread,
    object: ObjectReference,
    edge_visitor: &mut EV,
  ) {
    todo!()
  }

  fn notify_initial_thread_scan_complete(partial_scan: bool, tls: mmtk::util::VMWorkerThread) {
    todo!()
  }

  fn scan_roots_in_mutator_thread(
    tls: mmtk::util::VMWorkerThread,
    mutator: &'static mut mmtk::Mutator<MMTKHeap>,
    factory: impl mmtk::vm::RootsWorkFactory<<MMTKHeap as VMBinding>::VMEdge>,
  ) {
    todo!()
  }

  fn scan_vm_specific_roots(
    tls: mmtk::util::VMWorkerThread,
    factory: impl mmtk::vm::RootsWorkFactory<<MMTKHeap as VMBinding>::VMEdge>,
  ) {
    todo!()
  }

  fn supports_return_barrier() -> bool {
    todo!()
  }

  fn prepare_for_roots_re_scanning() {
    todo!()
  }
}

pub struct MMTKObjectModel {}

/// https://github.com/mmtk/mmtk-openjdk/blob/master/mmtk/src/object_model.rs
impl ObjectModel<MMTKHeap> for MMTKObjectModel {
  const GLOBAL_LOG_BIT_SPEC: mmtk::vm::VMGlobalLogBitSpec = unimplemented!();

  const LOCAL_FORWARDING_POINTER_SPEC: mmtk::vm::VMLocalForwardingPointerSpec = unimplemented!();

  const LOCAL_FORWARDING_BITS_SPEC: mmtk::vm::VMLocalForwardingBitsSpec = unimplemented!();

  const LOCAL_MARK_BIT_SPEC: mmtk::vm::VMLocalMarkBitSpec = unimplemented!();

  const LOCAL_LOS_MARK_NURSERY_SPEC: mmtk::vm::VMLocalLOSMarkNurserySpec = unimplemented!();

  const OBJECT_REF_OFFSET_LOWER_BOUND: isize = unimplemented!();

  fn copy(
    from: ObjectReference,
    semantics: mmtk::util::copy::CopySemantics,
    copy_context: &mut mmtk::util::copy::GCWorkerCopyContext<MMTKHeap>,
  ) -> ObjectReference {
    todo!()
  }

  fn copy_to(
    from: ObjectReference,
    to: ObjectReference,
    region: mmtk::util::Address,
  ) -> mmtk::util::Address {
    todo!()
  }

  fn get_reference_when_copied_to(
    from: ObjectReference,
    to: mmtk::util::Address,
  ) -> ObjectReference {
    todo!()
  }

  fn get_current_size(object: ObjectReference) -> usize {
    todo!()
  }

  fn get_size_when_copied(object: ObjectReference) -> usize {
    todo!()
  }

  fn get_align_when_copied(object: ObjectReference) -> usize {
    todo!()
  }

  fn get_align_offset_when_copied(object: ObjectReference) -> usize {
    todo!()
  }

  fn get_type_descriptor(reference: ObjectReference) -> &'static [i8] {
    todo!()
  }

  fn ref_to_object_start(object: ObjectReference) -> mmtk::util::Address {
    todo!()
  }

  fn ref_to_header(object: ObjectReference) -> mmtk::util::Address {
    todo!()
  }

  fn ref_to_address(object: ObjectReference) -> mmtk::util::Address {
    todo!()
  }

  fn address_to_ref(addr: mmtk::util::Address) -> ObjectReference {
    todo!()
  }

  fn dump_object(object: ObjectReference) {
    todo!()
  }
}

pub struct MMTKActivePlan {}

/// https://github.com/mmtk/mmtk-openjdk/blob/master/mmtk/src/active_plan.rs
impl ActivePlan<MMTKHeap> for MMTKActivePlan {
  fn is_mutator(tls: mmtk::util::VMThread) -> bool {
    todo!()
  }

  fn mutator(tls: mmtk::util::VMMutatorThread) -> &'static mut mmtk::Mutator<MMTKHeap> {
    todo!()
  }

  fn mutators<'a>() -> Box<dyn Iterator<Item = &'a mut mmtk::Mutator<MMTKHeap>> + 'a> {
    todo!()
  }

  fn number_of_mutators() -> usize {
    todo!()
  }
}

pub struct MMTKCollection {}

/// https://github.com/mmtk/mmtk-openjdk/blob/master/mmtk/src/collection.rs
impl Collection<MMTKHeap> for MMTKCollection {
  fn stop_all_mutators<F>(tls: mmtk::util::VMWorkerThread, mutator_visitor: F)
  where
    F: FnMut(&'static mut mmtk::Mutator<MMTKHeap>),
  {
    todo!()
  }

  fn resume_mutators(tls: mmtk::util::VMWorkerThread) {
    todo!()
  }

  fn block_for_gc(tls: mmtk::util::VMMutatorThread) {
    todo!()
  }

  fn spawn_gc_thread(tls: mmtk::util::VMThread, ctx: mmtk::vm::GCThreadContext<MMTKHeap>) {
    todo!()
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MMTKEdge {
  pub addr: Address,
}

impl MMTKEdge {
  pub const LOG_BYTES_IN_EDGE: usize = { 3 };
  pub const BYTES_IN_EDGE: usize = 1 << Self::LOG_BYTES_IN_EDGE;
}

impl From<Address> for MMTKEdge {
  fn from(value: Address) -> Self {
    Self { addr: value }
  }
}

/// https://github.com/mmtk/mmtk-openjdk/blob/master/mmtk/src/edges.rs
impl Edge for MMTKEdge {
  fn load(&self) -> ObjectReference {
    todo!()
  }

  fn store(&self, object: ObjectReference) {
    todo!()
  }
}

// A range of MMTKEdge, usually used for arrays.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MMTKEdgeRange {
  range: Range<MMTKEdge>,
}

impl From<Range<Address>> for MMTKEdgeRange {
  fn from(value: Range<Address>) -> Self {
    Self {
      range: Range {
        start: value.start.into(),
        end: value.end.into(),
      },
    }
  }
}

pub struct MMTKEdgeRangeIterator {
  cursor: Address,
  limit: Address,
}

impl Iterator for MMTKEdgeRangeIterator {
  type Item = MMTKEdge;

  fn next(&mut self) -> Option<Self::Item> {
    if self.cursor >= self.limit {
      None
    } else {
      let edge = self.cursor;
      self.cursor += MMTKEdge::BYTES_IN_EDGE;
      Some(edge.into())
    }
  }
}

impl From<MMTKEdgeRange> for Range<Address> {
  fn from(value: MMTKEdgeRange) -> Self {
    value.range.start.addr..value.range.end.addr
  }
}

/// https://github.com/mmtk/mmtk-openjdk/blob/master/mmtk/src/edges.rs
impl MemorySlice for MMTKEdgeRange {
  type Edge = MMTKEdge;
  type EdgeIterator = MMTKEdgeRangeIterator;

  fn iter_edges(&self) -> Self::EdgeIterator {
    MMTKEdgeRangeIterator {
      cursor: self.range.start.addr,
      limit: self.range.end.addr,
    }
  }

  fn object(&self) -> Option<ObjectReference> {
    None
  }

  fn start(&self) -> Address {
    self.range.start.addr
  }

  fn bytes(&self) -> usize {
    self.range.end.addr - self.range.start.addr
  }

  fn copy(src: &Self, tgt: &Self) {
    debug_assert_eq!(src.bytes(), tgt.bytes());
    // Raw memory copy
    debug_assert_eq!(
      src.bytes() & ((1 << LOG_BYTES_IN_WORD) - 1),
      0,
      "bytes are not a multiple of words"
    );
    Range::<Address>::copy(&src.clone().into(), &tgt.clone().into())
  }
}
