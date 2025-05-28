use page_table_entry::MappingFlags;
use x86::{controlregs::cr2, irq::*};
use x86_64::structures::idt::PageFaultErrorCode;
use crate::trap::PAGE_FAULT;
use super::context::TrapFrame;

core::arch::global_asm!(include_str!("trap.S"));

const IRQ_VECTOR_START: u8 = 0x20;
const IRQ_VECTOR_END: u8 = 0xff;

fn handle_page_fault(tf: &TrapFrame) {
    let access_flags = err_code_to_flags(tf.error_code)
        .unwrap_or_else(|e| panic!("Invalid #PF error code: {:#x}", e));
    let vaddr = va!(unsafe { cr2() });
    debug!("Page Fault Handle Function number: {}", PAGE_FAULT.len());
    debug!("Page Fault @ {:#x}, fault_vaddr={:#x}, error_code={:#x} ({:?})",
           tf.rip, vaddr, tf.error_code, access_flags);
    if !handle_trap!(PAGE_FAULT, vaddr, access_flags, tf.is_user()) {
        panic!(
            "Unhandled {} #PF @ {:#x}, fault_vaddr={:#x}, error_code={:#x} ({:?}):\n{:#x?}",
            if tf.is_user() { "user" } else { "kernel" },
            tf.rip,
            vaddr,
            tf.error_code,
            access_flags,
            tf,
        );
    }
}

#[no_mangle]
fn x86_trap_handler(tf: &TrapFrame) {
    let vector = tf.vector as u8;
    match vector as u8 {
        DIVIDE_ERROR_VECTOR => panic!("#DE (Divide Error) @ {:#x}", tf.rip),
        DEBUG_VECTOR => debug!("#DB (Debug) @ {:#x}", tf.rip),
        NONMASKABLE_INTERRUPT_VECTOR => panic!("Non-Maskable Interrupt @ {:#x}", tf.rip),
        BREAKPOINT_VECTOR => debug!("#BP (Breakpoint) @ {:#x}", tf.rip),
        OVERFLOW_VECTOR => panic!("#OF (Overflow) @ {:#x}", tf.rip),
        BOUND_RANGE_EXCEEDED_VECTOR => panic!("#BR (Bound Range Exceeded) @ {:#x}", tf.rip),
        INVALID_OPCODE_VECTOR => panic!("#UD (Invalid Opcode) @ {:#x}", tf.rip),
        DEVICE_NOT_AVAILABLE_VECTOR => panic!("#NM (Device Not Available) @ {:#x}", tf.rip),
        DOUBLE_FAULT_VECTOR => panic!("#DF (Double Fault) @ {:#x}", tf.rip),
        COPROCESSOR_SEGMENT_OVERRUN_VECTOR => {
            panic!("Coprocessor Segment Overrun (legacy) @ {:#x}", tf.rip)
        }
        INVALID_TSS_VECTOR => panic!("#TS (Invalid TSS) @ {:#x}", tf.rip),
        SEGMENT_NOT_PRESENT_VECTOR => panic!("#NP (Segment Not Present) @ {:#x}", tf.rip),
        STACK_SEGEMENT_FAULT_VECTOR => panic!("#SS (Stack Segment Fault) @ {:#x}", tf.rip),
        GENERAL_PROTECTION_FAULT_VECTOR => panic!(
            "#GP (General Protection Fault) @ {:#x}, error_code={:#x}:\n{:#x?}",
            tf.rip, tf.error_code, tf
        ),
        PAGE_FAULT_VECTOR => handle_page_fault(tf),
        X87_FPU_VECTOR => panic!("#MF (x87 Floating Point Exception) @ {:#x}", tf.rip),
        ALIGNMENT_CHECK_VECTOR => panic!("#AC (Alignment Check) @ {:#x}", tf.rip),
        MACHINE_CHECK_VECTOR => panic!("#MC (Machine Check) @ {:#x}", tf.rip),
        SIMD_FLOATING_POINT_VECTOR => panic!("#XF (SIMD Floating Point Exception) @ {:#x}", tf.rip),
        VIRTUALIZATION_VECTOR => panic!("#VE (Virtualization Exception) @ {:#x}", tf.rip),
        IRQ_VECTOR_START..=IRQ_VECTOR_END => {
            handle_trap!(IRQ, vector as usize);
        }

        _ => panic!(
            "Unhandled exception vector={} ({}) @ {:#x}, error_code={:#x}:\n{:#x?}",
            tf.vector,
            vec_to_str(tf.vector),
            tf.rip,
            tf.error_code,
            tf
        ),
    }
}

fn vec_to_str(vec: u64) -> &'static str {
    if vec < 32 {
        EXCEPTIONS[vec as usize].mnemonic
    } else {
        "Unknown"
    }
}

fn err_code_to_flags(err_code: u64) -> Result<MappingFlags, u64> {
    let code = PageFaultErrorCode::from_bits_truncate(err_code);
    let reserved_bits = (PageFaultErrorCode::CAUSED_BY_WRITE
        | PageFaultErrorCode::USER_MODE
        | PageFaultErrorCode::INSTRUCTION_FETCH)
        .complement();
    if code.intersects(reserved_bits) {
        Err(err_code)
    } else {
        let mut flags = MappingFlags::empty();
        if code.contains(PageFaultErrorCode::CAUSED_BY_WRITE) {
            flags |= MappingFlags::WRITE;
        } else {
            flags |= MappingFlags::READ;
        }
        if code.contains(PageFaultErrorCode::USER_MODE) {
            flags |= MappingFlags::USER;
        }
        if code.contains(PageFaultErrorCode::INSTRUCTION_FETCH) {
            flags |= MappingFlags::EXECUTE;
        }
        Ok(flags)
    }
}
