//! Description tables (per-CPU GDT, per-CPU ISS, IDT)

use crate::arch::{GdtStruct, IdtStruct, IDT, TSS, GDT};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::DescriptorTablePointer;

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
    init_percpu();
}


/// Initializes IDT, GDT on secondary CPUs.
#[cfg(feature = "smp")]
pub(super) fn init_secondary() {
    init_percpu();
}
