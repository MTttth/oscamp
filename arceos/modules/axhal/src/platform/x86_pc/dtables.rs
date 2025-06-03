//! Description tables (per-CPU GDT, per-CPU ISS, IDT)

use crate::arch::{GdtStruct, IdtStruct};
use x86_64::structures::tss::TaskStateSegment;
use lazyinit::LazyInit;
use x86_64::structures::DescriptorTablePointer;
static IDT: LazyInit<IdtStruct> = LazyInit::new();

#[percpu::def_percpu]
static TSS: LazyInit<TaskStateSegment> = LazyInit::new();

#[percpu::def_percpu]
static GDT: LazyInit<GdtStruct> = LazyInit::new();

fn init_percpu() {
    unsafe {
        unsafe { IDT.load() };
        let tss = TSS.current_ref_mut_raw();
        let gdt = GDT.current_ref_mut_raw();
        tss.init_once(TaskStateSegment::new());
        gdt.init_once(GdtStruct::new(tss));
        gdt.load();
        gdt.load_tss();
    }
}

/// Initializes IDT, GDT on the primary CPU.
pub(super) fn init_primary() {
    axlog::ax_println!("\nInitialize IDT & GDT...");
    IDT.init_once(IdtStruct::new());
    let idtr = IDT.pointer();
    let idtr_base = idtr.base;
    let idtr_limit = idtr.limit;
    axlog::ax_println!("→ [Debug] IDTR.base = {:#x}, limit = {:#x}", idtr_base, idtr_limit);
    // axlog::ax_println!("IDT :{:?}", IDT);
    init_percpu();
    let gdt_struct = unsafe{ GDT.current_ref_raw() };
    let gdtr = gdt_struct.pointer();
    let gdt_base = gdtr.base;
    let gdt_limit = gdtr.limit;
    axlog::ax_println!("→ [Debug] GDT.base = {:#x}, limit = {:#x}", gdt_base, gdt_limit);
}


/// Initializes IDT, GDT on secondary CPUs.
#[cfg(feature = "smp")]
pub(super) fn init_secondary() {
    init_percpu();
}
