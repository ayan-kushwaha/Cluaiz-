//! 🏛️ Silicon Kernel: ISA Probing (Assembly Level)
//! Manual register queries via inline assembly to bypass OS abstraction.
//! Bores into the bare-metal to identify specialized silicon accelerators.

/// Queries various CPU control registers using Inline Assembly.
/// Enforces the V8 'Nichoding' Protocol (Full hardware utilization).
pub struct IsaProbe;

impl IsaProbe {
    /// Queries CPUID Leaf 7 Sub-leaf 0 for AMX and AVX-512 flags.
    /// Zero-Latency: Pure register read.
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn query_x86_capabilities() -> (u32, u32) {
        // CPUID inputs go in via eax=7, ecx=0 clobbers.
        // We use lateout to tell the borrow checker: we only care about output values.
        let eax_in: u32 = 7;
        let ebx: u32;
        let edx: u32;

        std::arch::asm!(
            "push rbx",
            "mov eax, {0:e}",
            "xor ecx, ecx",
            "cpuid",
            "mov {1:e}, ebx",
            "pop rbx",
            in(reg) eax_in,
            out(reg) ebx,
            lateout("edx") edx,
            out("eax") _,
            out("ecx") _,
        );

        (ebx, edx) // Contains AVX512 and AMX bits
    }

    /// Queries ARM ID_AA64ISAR0_EL1 register for Dot Product and MatMul support.
    #[cfg(target_arch = "aarch64")]
    pub unsafe fn query_arm_capabilities() -> u64 {
        let val: u64;
        // MRS = Move System Register to General Purpose Register
        std::arch::asm!(
            "mrs {0}, id_aa64isar0_el1",
            out(reg) val,
        );
        val
    }
}
