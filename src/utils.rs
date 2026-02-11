use asr::{
    Address, Process,
    game_engine::unity::il2cpp::{Image, Module, UnityPointer},
};
use bytemuck::CheckedBitPattern;

pub fn ptrpath<const N: usize>(cls: &'static str, path: [&'static str; N]) -> UnityPointer<N> {
    return UnityPointer::new(cls, 0, path.as_ref());
}

pub fn ptrpath1<const N: usize>(cls: &'static str, path: [&'static str; N]) -> UnityPointer<N> {
    return UnityPointer::new(cls, 1, path.as_ref());
}

pub struct Assembly {
    pub module: Module,
    pub image: Image,
}

pub trait UnityPointerExt<const N: usize> {
    fn read<T: CheckedBitPattern>(&self, process: &Process, asm: &Assembly) -> Option<T>;

    fn bool(&self, process: &Process, asm: &Assembly) -> Option<bool> {
        return self.read::<bool>(process, asm);
    }

    fn f64(&self, process: &Process, asm: &Assembly) -> Option<f64> {
        return self.read::<f64>(process, asm);
    }

    fn u64(&self, process: &Process, asm: &Assembly) -> Option<u64> {
        return self.read::<u64>(process, asm);
    }

    fn addr(&self, process: &Process, asm: &Assembly) -> Option<Address>;
}

impl<const N: usize> UnityPointerExt<N> for UnityPointer<N> {
    fn read<T: CheckedBitPattern>(&self, process: &Process, asm: &Assembly) -> Option<T> {
        return self.deref(process, &asm.module, &asm.image).ok();
    }

    fn addr(&self, process: &Process, asm: &Assembly) -> Option<Address> {
        return self.deref_offsets(process, &asm.module, &asm.image).ok();
    }
}
