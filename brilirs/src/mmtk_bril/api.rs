use crate::allocator;
use crate::allocator::Value;
use crate::mmtk_bril::mmtk;
use crate::mmtk_bril::BrilGC;
use crate::mmtk_bril::SINGLETON;
use mmtk::memory_manager;
use mmtk::scheduler::GCWorker;
use mmtk::util::opaque_pointer::*;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::Finalizable;
use mmtk::AllocationSemantics;
use mmtk::MMTKBuilder;
use mmtk::Mutator;

use super::MMTKBrilPointer;

// This file exposes MMTk Rust API to the native code. This is not an exhaustive list of all the APIs.
// Most commonly used APIs are listed in https://docs.mmtk.io/api/mmtk/memory_manager/index.html. The binding can expose them here.

pub fn set_fixed_heap_size(builder: &mut MMTKBuilder, heap_size: usize) -> bool {
  builder
    .options
    .gc_trigger
    .set(mmtk::util::options::GCTriggerSelector::FixedHeapSize(
      heap_size,
    ))
}

pub fn init(builder: &mut MMTKBuilder) {
  // Create MMTK instance.
  let mmtk = memory_manager::mmtk_init::<BrilGC>(&builder);

  // Set SINGLETON to the instance.
  SINGLETON.set(mmtk).unwrap_or_else(|_| {
    panic!("Failed to set SINGLETON");
  });
}

pub fn mmtk_bind_mutator(tls: VMMutatorThread) -> Box<Mutator<BrilGC>> {
  memory_manager::bind_mutator(mmtk(), tls)
}

pub fn mmtk_destroy_mutator(mut mutator: Box<Mutator<BrilGC>>) {
  // notify mmtk-core about destroyed mutator
  memory_manager::destroy_mutator(&mut mutator);
}

pub fn mmtk_alloc(
  mutator: &mut Mutator<BrilGC>,
  size: usize,
  align: usize,
  offset: usize,
  mut semantics: AllocationSemantics,
) -> Address {
  // This just demonstrates that the binding should check against `max_non_los_default_alloc_bytes` to allocate large objects.
  // In pratice, a binding may want to lift this code to somewhere in the runtime where the allocated bytes is constant so
  // they can statically know if a normal allocation or a large object allocation is needed.
  if size
    >= mmtk()
      .get_plan()
      .constraints()
      .max_non_los_default_alloc_bytes
  {
    semantics = AllocationSemantics::Los;
  }
  memory_manager::alloc::<BrilGC>(mutator, size, align, offset, semantics)
}

pub fn bril_alloc(mutator: &mut Mutator<BrilGC>, arr_size: usize) -> Address {
  let size = std::mem::size_of::<allocator::Value<MMTKBrilPointer>>() * arr_size;

  mmtk_alloc(mutator, size, 8, 0, mmtk::AllocationSemantics::Default)
}

pub fn bril_load(addr: Address) -> Value<MMTKBrilPointer> {
  unsafe { addr.load::<i32>() };
  todo!()
}

pub fn bril_store() {
  todo!()
}

pub fn mmtk_post_alloc(
  mutator: &mut Mutator<BrilGC>,
  refer: ObjectReference,
  bytes: usize,
  mut semantics: AllocationSemantics,
) {
  // This just demonstrates that the binding should check against `max_non_los_default_alloc_bytes` to allocate large objects.
  // In pratice, a binding may want to lift this code to somewhere in the runtime where the allocated bytes is constant so
  // they can statically know if a normal allocation or a large object allocation is needed.
  if bytes
    >= mmtk()
      .get_plan()
      .constraints()
      .max_non_los_default_alloc_bytes
  {
    semantics = AllocationSemantics::Los;
  }
  memory_manager::post_alloc::<BrilGC>(mutator, refer, bytes, semantics)
}

pub fn mmtk_start_worker(tls: VMWorkerThread, worker: Box<GCWorker<BrilGC>>) {
  memory_manager::start_worker::<BrilGC>(mmtk(), tls, worker)
}

pub fn mmtk_initialize_collection(tls: VMThread) {
  memory_manager::initialize_collection(mmtk(), tls)
}

pub fn mmtk_used_bytes() -> usize {
  memory_manager::used_bytes(mmtk())
}

pub fn mmtk_free_bytes() -> usize {
  memory_manager::free_bytes(mmtk())
}

pub fn mmtk_total_bytes() -> usize {
  memory_manager::total_bytes(mmtk())
}

pub fn mmtk_is_live_object(object: ObjectReference) -> bool {
  memory_manager::is_live_object(object)
}

pub fn mmtk_will_never_move(object: ObjectReference) -> bool {
  !object.is_movable()
}

pub fn mmtk_is_in_mmtk_spaces(object: ObjectReference) -> bool {
  memory_manager::is_in_mmtk_spaces(object)
}

pub fn mmtk_is_mapped_address(address: Address) -> bool {
  memory_manager::is_mapped_address(address)
}

pub fn mmtk_handle_user_collection_request(tls: VMMutatorThread) {
  memory_manager::handle_user_collection_request::<BrilGC>(mmtk(), tls);
}

pub fn mmtk_add_weak_candidate(reff: ObjectReference) {
  memory_manager::add_weak_candidate(mmtk(), reff)
}

pub fn mmtk_add_soft_candidate(reff: ObjectReference) {
  memory_manager::add_soft_candidate(mmtk(), reff)
}

pub fn mmtk_add_phantom_candidate(reff: ObjectReference) {
  memory_manager::add_phantom_candidate(mmtk(), reff)
}

pub fn mmtk_harness_begin(tls: VMMutatorThread) {
  memory_manager::harness_begin(mmtk(), tls)
}

pub fn mmtk_harness_end() {
  memory_manager::harness_end(mmtk())
}

pub fn mmtk_starting_heap_address() -> Address {
  memory_manager::starting_heap_address()
}

pub fn mmtk_last_heap_address() -> Address {
  memory_manager::last_heap_address()
}

pub fn mmtk_malloc(size: usize) -> Address {
  memory_manager::malloc(size)
}

pub fn mmtk_calloc(num: usize, size: usize) -> Address {
  memory_manager::calloc(num, size)
}

pub fn mmtk_realloc(addr: Address, size: usize) -> Address {
  memory_manager::realloc(addr, size)
}

pub fn mmtk_free(addr: Address) {
  memory_manager::free(addr)
}

pub fn mmtk_init_test() {
  // We demonstrate the main workflow to initialize MMTk, create mutators and allocate objects.
  let mut builder = MMTKBuilder::new();

  // Set option by value using extern "C" wrapper.
  let success = set_fixed_heap_size(&mut builder, 1048576);
  assert!(success);

  // Set option by value.  We set the the option direcly using `MMTKOption::set`. Useful if
  // the VM binding wants to set options directly, or if the VM binding has its own format for
  // command line arguments.
  let name = "plan";
  let val = "NoGC";
  let success = builder.set_option(name, val);
  assert!(success);

  // Init MMTk
  init(&mut builder);

  // Create an MMTk mutator
  let tls = VMMutatorThread(VMThread(OpaquePointer::UNINITIALIZED)); // FIXME: Use the actual thread pointer or identifier
  let mut mutator = mmtk_bind_mutator(tls);

  // Do an allocation
  let addr = mmtk_alloc(&mut mutator, 16, 8, 0, mmtk::AllocationSemantics::Default);
  assert!(!addr.is_zero());

  unsafe { addr.store(1) };

  let res = unsafe { addr.load::<i32>() };
  println!("Value at address: {}", res);

  // Turn the allocation address into the object reference.
  let obj = BrilGC::object_start_to_ref(addr);

  // Post allocation
  mmtk_post_alloc(&mut mutator, obj, 16, mmtk::AllocationSemantics::Default);

  // If the thread quits, destroy the mutator.
  mmtk_destroy_mutator(mutator);
}
