use crate::spec::{Arch, StackProbeType, Target, Os, Env, LinkerFlavor, TargetMetadata, TargetOptions};

pub(crate) fn target() -> Target {
    Target {
        llvm_target: "x86_64-unknown-vespera".into(),
        metadata: TargetMetadata {
            description: Some("VesperaOS userspace (x86_64)".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: Some(true),
        },
        pointer_width: 64,
        arch: Arch::X86_64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128".into(),
        options: TargetOptions {
            os: Os::Vespera,
            env: Env::Unspecified,
            linker_flavor: LinkerFlavor::Gnu(crate::spec::Cc::No, crate::spec::Lld::Yes),
            linker: Some("rust-lld".into()),
            vendor: "unknown".into(),
            max_atomic_width: Some(64),
            stack_probes: StackProbeType::Inline,
            has_thread_local: true,
            tls_model: crate::spec::TlsModel::InitialExec,
            position_independent_executables: true,
            static_position_independent_executables: true,
            relocation_model: crate::spec::RelocModel::Pic,
            panic_strategy: crate::spec::PanicStrategy::Abort,
            ..Default::default()
        },
    }
}