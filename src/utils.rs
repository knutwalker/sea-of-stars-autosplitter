#![allow(dead_code)]

use core::{fmt, marker::PhantomData};

use asr::{
    Address, Address64, Process,
    game_engine::unity::il2cpp::{Image, Module, UnityPointer},
    string::ArrayString,
};
use bytemuck::{AnyBitPattern, CheckedBitPattern};

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

    fn ptr<T: 'static>(&self, process: &Process, asm: &Assembly) -> Option<Pointer<T>> {
        let addr = self.addr(process, asm)?;
        Some(Pointer::from(addr))
    }
}

impl<const N: usize> UnityPointerExt<N> for UnityPointer<N> {
    fn read<T: CheckedBitPattern>(&self, process: &Process, asm: &Assembly) -> Option<T> {
        return self.deref(process, &asm.module, &asm.image).ok();
    }

    fn addr(&self, process: &Process, asm: &Assembly) -> Option<Address> {
        return self.deref_offsets(process, &asm.module, &asm.image).ok();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnumSet<T>(u128, PhantomData<T>);

pub trait EnumSetMember {
    fn ordinal(&self) -> Option<u8>;
}

impl<T: EnumSetMember> EnumSet<T> {
    pub const fn empty() -> Self {
        Self(0, PhantomData)
    }

    pub fn insert(&mut self, item: &T) -> bool {
        let Some(ord) = item.ordinal() else {
            return false;
        };
        if ord >= 128 {
            return false;
        }

        let mask = 1_u128 << ord;
        let previous = self.0 & mask;
        self.0 |= mask;
        return previous == 0;
    }

    pub fn contains(&mut self, item: &T) -> bool {
        let Some(ord) = item.ordinal() else {
            return false;
        };
        if ord >= 128 {
            return false;
        }

        let mask = 1_u128 << ord;
        return (self.0 & mask) == mask;
    }

    pub fn into_bits(self) -> u128 {
        return self.0;
    }
}

/// Trait for things that can read data from memory.
pub trait MemReader: Sized {
    /// Reads a value from memory.
    fn read<T: CheckedBitPattern, A: Into<Address>>(&self, addr: A) -> Option<T>;
}

impl MemReader for Process {
    fn read<T: CheckedBitPattern, A: Into<Address>>(&self, addr: A) -> Option<T> {
        self.read(addr).ok()
    }
}

/// A pointer to a value in memory.
/// This type has the same memory layout as an [`Address64`] and
/// can be used in place of it, typically in classes derived when
/// the `derive` feature is enabled and used.
/// Using this type instead of [`Address64`] can give a bit more
/// type safety.
#[repr(C)]
pub struct Pointer<T> {
    address: Address64,
    _t: PhantomData<T>,
}

impl<T: CheckedBitPattern> Pointer<T> {
    /// Read a value from memory by following this pointer.
    pub fn read<R: MemReader>(self, reader: &R) -> Option<T> {
        if self.address.is_null() {
            None
        } else {
            reader.read(self.address)
        }
    }
}

impl<T> Pointer<T> {
    pub const NULL: Self = Pointer {
        address: Address64::NULL,
        _t: PhantomData,
    };

    /// Return the address of this pointer.
    pub const fn address(self) -> Address64 {
        self.address
    }

    /// Return the address of this pointer as generic `Address`.
    pub fn addr(self) -> Address {
        self.address.into()
    }
}

impl Pointer<CSString> {
    pub fn chars<R: MemReader>(self, reader: &R) -> Option<impl Iterator<Item = char> + '_> {
        let string = self.read(reader)?;
        let start = self.address() + CSString::DATA;
        let end = start + u64::from((size_of::<u16>() / size_of::<u8>()) as u32 * string.size);

        let utf16 = ArrayIter {
            pos: start,
            end,
            reader,
            _t: PhantomData::<u16>,
        };
        Some(char::decode_utf16(utf16).map(|o| o.unwrap_or(char::REPLACEMENT_CHARACTER)))
    }

    pub fn to_string<R: MemReader, const CAP: usize>(self, reader: &R) -> Option<ArrayString<CAP>> {
        let chars = self.chars(reader)?;
        let mut s = ArrayString::new();
        for c in chars {
            match s.try_push(c) {
                Ok(()) => {}
                Err(_) => break,
            }
        }
        Some(s)
    }

    // #[cfg(feature = "alloc")]
    // pub fn to_std_string<R: MemReader>(self, reader: &R) -> Option<::alloc::string::String> {
    //     Some(self.chars(reader)?.collect())
    // }
}

impl<T: CheckedBitPattern + 'static> Pointer<Array<T>> {
    pub fn iter<R: MemReader>(self, reader: &R) -> Option<ArrayIter<'_, T, R>> {
        let array = self.read(reader)?;
        let start = self.address() + Array::<T>::DATA;
        let end = start + (size_of::<T>() * array.size as usize) as u64;

        Some(ArrayIter {
            pos: start,
            end,
            reader,
            _t: PhantomData,
        })
    }

    pub fn get<R: MemReader>(self, reader: &R, index: usize) -> Option<T> {
        let array = self.read(reader)?;
        if index >= array.size as usize {
            return None;
        }
        let offset = self.address() + Array::<T>::DATA + (index * size_of::<T>()) as u64;
        reader.read(offset)
    }
}

impl<T: CheckedBitPattern + 'static> Pointer<List<T>> {
    pub fn iter<R: MemReader>(self, reader: &R) -> Option<impl Iterator<Item = T> + '_> {
        let list = self.read(reader)?;
        Some(list.items.iter(reader)?.take(list.size as _))
    }
}

impl<K: AnyBitPattern + 'static, V: AnyBitPattern + 'static> Pointer<Map<K, V>> {
    pub fn iter<R: MemReader>(self, reader: &R) -> Option<impl Iterator<Item = (K, V)> + '_> {
        let map = self.read(reader)?;
        Some(
            map.entries
                .iter(reader)?
                .filter(|o| o._hash != 0 || o._next != 0)
                .take(map.size as _)
                .map(|o| (o.key, o.value)),
        )
    }
}

impl<T> From<Pointer<T>> for Address {
    fn from(ptr: Pointer<T>) -> Self {
        ptr.address.into()
    }
}

impl<T> From<Pointer<T>> for Address64 {
    fn from(ptr: Pointer<T>) -> Self {
        ptr.address
    }
}

impl<T> From<Address64> for Pointer<T> {
    fn from(addr: Address64) -> Self {
        Self {
            address: addr,
            _t: PhantomData,
        }
    }
}

impl<T: 'static> From<Address> for Pointer<T> {
    fn from(addr: Address) -> Self {
        bytemuck::cast(addr.value())
    }
}

impl<T> fmt::Debug for Pointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pointer")
            .field("address", &self.address)
            .field("type", &core::any::type_name::<T>())
            .finish()
    }
}

// This is a manual implementation and not derived because the derive
// implementation would add a `T: Copy` bound, which is not required.
impl<T> ::core::marker::Copy for Pointer<T> {}

// This is a manual implementation and not derived because the derive
// implementation would add a `T: Clone` bound, which is not required.
impl<T> ::core::clone::Clone for Pointer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// This is a manual implementation and not derived because the derive
// macro would add a `T: AnyBitPattern` bound, which is not required.
//
// SAFETY:
// Similar to raw pointers, a pointer is valid for any bit pattern
// Dereferencing the pointer is not, though.
unsafe impl<T: 'static> ::bytemuck::AnyBitPattern for Pointer<T> {}

// This is a manual implementation and not derived because the derive
// macro would add a `T: Zeroable` bound, which is not required.
//
// SAFETY:
// A zeroed pointer is the null pointer, and it is a valid pointer.
// It must not be derreferenced, though.
unsafe impl<T: 'static> ::bytemuck::Zeroable for Pointer<T> {}

#[repr(C)]
pub struct CSString {
    _type_id: [u32; 2],
    _header: [u32; 2],
    size: u32,
}

impl CSString {
    const DATA: u64 = 0x14;
}

const _: () = {
    assert!(size_of::<CSString>() == CSString::DATA as usize);
};

impl CSString {
    pub const fn size(&self) -> u32 {
        self.size
    }
}

impl fmt::Debug for CSString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CSString")
            .field("size", &self.size)
            .finish()
    }
}
// This is a manual implementation and not derived because the derive
// implementation would add a `T: Copy` bound, which is not required.
impl ::core::marker::Copy for CSString {}

// This is a manual implementation and not derived because the derive
// implementation would add a `T: Clone` bound, which is not required.
impl ::core::clone::Clone for CSString {
    fn clone(&self) -> Self {
        *self
    }
}

// This is a manual implementation and not derived because the derive
// macro would add a `T: AnyBitPattern` bound, which is not required.
unsafe impl ::bytemuck::AnyBitPattern for CSString {}

// This is a manual implementation and not derived because the derive
// macro would add a `T: Zeroable` bound, which is not required.
unsafe impl ::bytemuck::Zeroable for CSString {}

#[repr(C)]
pub struct Array<T> {
    _type_id: u64,
    _header: u64,
    _header2: u64,
    size: u32,
    _t: PhantomData<T>,
}

impl<T> Array<T> {
    const DATA: u64 = 0x20;
}

const _: () = {
    assert!(size_of::<Array<()>>() == Array::<()>::DATA as usize);
};

pub struct ArrayIter<'a, T, R> {
    pos: Address64,
    end: Address64,
    reader: &'a R,
    _t: PhantomData<T>,
}

impl<T: CheckedBitPattern, R: MemReader> Iterator for ArrayIter<'_, T, R> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.end {
            return None;
        }

        let item: T = self.reader.read(self.pos)?;

        self.pos = self.pos + size_of::<T>() as u64;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end.value().saturating_sub(self.pos.value()) as usize;
        (remaining, Some(remaining))
    }
}

impl<T> fmt::Debug for Array<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Array")
            .field("size", &self.size)
            .field("type", &core::any::type_name::<T>())
            .finish()
    }
}

// This is a manual implementation and not derived because the derive
// implementation would add a `T: Copy` bound, which is not required.
impl<T> ::core::marker::Copy for Array<T> {}

// This is a manual implementation and not derived because the derive
// implementation would add a `T: Clone` bound, which is not required.
impl<T> ::core::clone::Clone for Array<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// This is a manual implementation and not derived because the derive
// macro would add a `T: AnyBitPattern` bound, which is not required.
//
// SAFETY:
// While technically not any bit pattern is allowed, we are ignoring
// the C# object header internals, so for the purpose of this type
// they can indeed be anything.
unsafe impl<T: 'static> ::bytemuck::AnyBitPattern for Array<T> {}

// This is a manual implementation and not derived because the derive
// macro would add a `T: Zeroable` bound, which is not required.
//
// SAFETY:
// Similar to the logic for AnyBitPattern, we accept zeroed values
// because we only care about the size field and that one is ok
// to be zero.
unsafe impl<T: 'static> ::bytemuck::Zeroable for Array<T> {}

#[repr(C)]
pub struct List<T> {
    _type_id: u64,
    _header: u64,
    items: Pointer<Array<T>>,
    size: u32,
}

impl<T> fmt::Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("List")
            .field("items", &self.items)
            .field("size", &self.size)
            .finish()
    }
}

// This is a manual implementation and not derived because the derive
// implementation would add a `T: Copy` bound, which is not required.
impl<T> ::core::marker::Copy for List<T> {}

// This is a manual implementation and not derived because the derive
// implementation would add a `T: Clone` bound, which is not required.
impl<T> ::core::clone::Clone for List<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// This is a manual implementation and not derived because the derive
// macro would add a `T: AnyBitPattern` bound, which is not required.
unsafe impl<T: 'static> ::bytemuck::AnyBitPattern for List<T> {}

// This is a manual implementation and not derived because the derive
// macro would add a `T: Zeroable` bound, which is not required.
unsafe impl<T: 'static> ::bytemuck::Zeroable for List<T> {}

#[repr(C)]
pub struct Map<K, V> {
    _type_id: u64,
    _header: u64,
    _header_2: u64,
    entries: Pointer<Array<Entry<K, V>>>,
    pub size: u32,
}

#[derive(Copy, Clone, Debug, AnyBitPattern)]
#[repr(C)]
pub struct Entry<K, V> {
    _hash: u32,
    _next: u32,
    key: K,
    value: V,
}

impl<K, V> fmt::Debug for Map<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map")
            .field("entries", &self.entries)
            .field("size", &self.size)
            .finish()
    }
}

// This is a manual implementation and not derived because the derive
// implementation would add `K: Copy` and `V: Copy` bounds, which is
// not required.
impl<K, V> ::core::marker::Copy for Map<K, V> {}

// This is a manual implementation and not derived because the derive
// implementation would add `K: Clone` and `V: Clone` bounds, which is
// not required.
impl<K, V> ::core::clone::Clone for Map<K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

// This is a manual implementation and not derived because the derive
// macro would add `K: AnyBitPattern` and `V: AnyBitPattern` bounds,
// which is not required.
unsafe impl<K: 'static, V: 'static> ::bytemuck::AnyBitPattern for Map<K, V> {}

// This is a manual implementation and not derived because the derive
// macro would add `K: Zeroable` and `V: Zeroable` bounds, which is
// not required.
unsafe impl<K: 'static, V: 'static> ::bytemuck::Zeroable for Map<K, V> {}
