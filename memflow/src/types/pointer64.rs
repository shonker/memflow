/*!
64-bit Pointer abstraction.
*/

use crate::dataview::Pod;
use crate::error::PartialResult;
use crate::mem::VirtualMemory;
use crate::types::{Address, ByteSwap};

use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem::size_of;
use std::{cmp, fmt, hash, ops};

/// This type can be used in structs that are being read from the target memory.
/// It holds a phantom type that can be used to describe the proper type of the pointer
/// and to read it in a more convenient way.
///
/// This module is a direct adaption of [CasualX's great IntPtr crate](https://github.com/CasualX/intptr).
///
/// Generally the generic Type should implement the Pod trait to be read into easily.
/// See [here](https://docs.rs/dataview/0.1.1/dataview/) for more information on the Pod trait.
///
/// # Examples
///
/// ```
/// use memflow::types::Pointer64;
/// use memflow::mem::VirtualMemory;
/// use memflow::dataview::Pod;
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Foo {
///     pub some_value: i64,
/// }
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Bar {
///     pub foo_ptr: Pointer64<Foo>,
/// }
///
/// fn read_foo_bar<T: VirtualMemory>(virt_mem: &mut T) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = bar.foo_ptr.deref(virt_mem).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::dummy::DummyMemory;
/// # use memflow::types::size;
/// # read_foo_bar(&mut DummyMemory::new_virt(size::mb(4), size::mb(2), &[]).0);
/// ```
///
/// ```
/// use memflow::types::Pointer64;
/// use memflow::mem::VirtualMemory;
/// use memflow::dataview::Pod;
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Foo {
///     pub some_value: i64,
/// }
///
/// #[repr(C)]
/// #[derive(Clone, Debug, Pod)]
/// struct Bar {
///     pub foo_ptr: Pointer64<Foo>,
/// }
///
/// fn read_foo_bar<T: VirtualMemory>(virt_mem: &mut T) {
///     let bar: Bar = virt_mem.virt_read(0x1234.into()).unwrap();
///     let foo = virt_mem.virt_read_ptr64(bar.foo_ptr).unwrap();
///     println!("value: {}", foo.some_value);
/// }
///
/// # use memflow::dummy::DummyMemory;
/// # use memflow::types::size;
/// # read_foo_bar(&mut DummyMemory::new_virt(size::mb(4), size::mb(2), &[]).0);
/// ```
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Pointer64<T: ?Sized = ()> {
    pub address: u64,
    phantom_data: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Pointer64<T> {
    const PHANTOM_DATA: PhantomData<fn() -> T> = PhantomData;

    /// A pointer64 with the value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// println!("pointer64: {}", Pointer64::<()>::NULL);
    /// ```
    pub const NULL: Pointer64<T> = Pointer64 {
        address: 0,
        phantom_data: PhantomData,
    };

    /// Returns a pointer64 with a value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// println!("pointer64: {}", Pointer64::<()>::null());
    /// ```
    pub const fn null() -> Self {
        Pointer64::NULL
    }

    /// Checks wether the pointer64 is zero or not.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// assert_eq!(Pointer64::<()>::null().is_null(), true);
    /// assert_eq!(Pointer64::<()>::from(0x1000u64).is_null(), false);
    /// ```
    pub const fn is_null(self) -> bool {
        self.address == 0
    }

    /// Converts the pointer64 to an Option that is None when it is null
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// assert_eq!(Pointer64::<()>::null().non_null(), None);
    /// assert_eq!(Pointer64::<()>::from(0x1000u64).non_null(), Some(Pointer64::from(0x1000u64)));
    /// ```
    #[inline]
    pub fn non_null(self) -> Option<Pointer64<T>> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }

    /// Converts the pointer64 into a `u32` value.
    ///
    /// # Remarks:
    ///
    /// This function internally uses `as u32` which can cause a wrap-around
    /// in case the internal 64-bit value does not fit the 32-bit `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<()>::from(0x1000u64);
    /// let ptr_u32: u32 = ptr.as_u32();
    /// assert_eq!(ptr_u32, 0x1000);
    /// ```
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.address as u32
    }

    /// Converts the pointer64 into a `u64` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<()>::from(0x1000u64);
    /// let ptr_u64: u64 = ptr.as_u64();
    /// assert_eq!(ptr_u64, 0x1000);
    /// ```
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.address as u64
    }

    /// Converts the pointer64 into a `usize` value.
    ///
    /// # Remarks:
    ///
    /// When compiling for a 32-bit architecture the size of `usize`
    /// is only 32-bit. Since this function internally uses `as usize` it can cause a wrap-around
    /// in case the internal 64-bit value does not fit in the 32-bit `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Pointer64;
    ///
    /// let ptr = Pointer64::<()>::from(0x1000u64);
    /// let ptr_usize: usize = ptr.as_usize();
    /// assert_eq!(ptr_usize, 0x1000);
    /// ```
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.address as usize
    }

    /// Returns the underlying raw u64 value of this pointer.
    #[deprecated = "use as_u64() instead"]
    pub const fn into_raw(self) -> u64 {
        self.address
    }
}

/// This function will deref the pointer directly into a Pod type.
impl<T: Pod + ?Sized> Pointer64<T> {
    pub fn deref_into<U: VirtualMemory>(self, mem: &mut U, out: &mut T) -> PartialResult<()> {
        mem.virt_read_ptr64_into(self, out)
    }
}

/// This function will return the Object this pointer is pointing towards.
impl<T: Pod + Sized> Pointer64<T> {
    pub fn deref<U: VirtualMemory>(self, mem: &mut U) -> PartialResult<T> {
        mem.virt_read_ptr64(self)
    }
}

impl<T> Pointer64<[T]> {
    pub const fn decay(self) -> Pointer64<T> {
        Pointer64 {
            address: self.address,
            phantom_data: Pointer64::<T>::PHANTOM_DATA,
        }
    }

    pub const fn at(self, i: usize) -> Pointer64<T> {
        let address = self.address + (i * size_of::<T>()) as u64;
        Pointer64 {
            address,
            phantom_data: Pointer64::<T>::PHANTOM_DATA,
        }
    }
}

impl<T: ?Sized> Copy for Pointer64<T> {}
impl<T: ?Sized> Clone for Pointer64<T> {
    #[inline(always)]
    fn clone(&self) -> Pointer64<T> {
        *self
    }
}
impl<T: ?Sized> Default for Pointer64<T> {
    #[inline(always)]
    fn default() -> Pointer64<T> {
        Pointer64::NULL
    }
}
impl<T: ?Sized> Eq for Pointer64<T> {}
impl<T: ?Sized> PartialEq for Pointer64<T> {
    #[inline(always)]
    fn eq(&self, rhs: &Pointer64<T>) -> bool {
        self.address == rhs.address
    }
}
impl<T: ?Sized> PartialOrd for Pointer64<T> {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Pointer64<T>) -> Option<cmp::Ordering> {
        self.address.partial_cmp(&rhs.address)
    }
}
impl<T: ?Sized> Ord for Pointer64<T> {
    #[inline(always)]
    fn cmp(&self, rhs: &Pointer64<T>) -> cmp::Ordering {
        self.address.cmp(&rhs.address)
    }
}
impl<T: ?Sized> hash::Hash for Pointer64<T> {
    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state)
    }
}
impl<T: ?Sized> AsRef<u64> for Pointer64<T> {
    #[inline(always)]
    fn as_ref(&self) -> &u64 {
        &self.address
    }
}
impl<T: ?Sized> AsMut<u64> for Pointer64<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut u64 {
        &mut self.address
    }
}

// From implementations
impl<T: ?Sized> From<u32> for Pointer64<T> {
    #[inline(always)]
    fn from(address: u32) -> Pointer64<T> {
        Pointer64 {
            address: address as u64,
            phantom_data: PhantomData,
        }
    }
}

impl<T: ?Sized> From<u64> for Pointer64<T> {
    #[inline(always)]
    fn from(address: u64) -> Pointer64<T> {
        Pointer64 {
            address,
            phantom_data: PhantomData,
        }
    }
}

impl<T: ?Sized> From<Address> for Pointer64<T> {
    #[inline(always)]
    fn from(address: Address) -> Pointer64<T> {
        Pointer64 {
            address: address.as_u64(),
            phantom_data: PhantomData,
        }
    }
}

// Into implementations
impl<T: ?Sized> From<Pointer64<T>> for Address {
    #[inline(always)]
    fn from(ptr: Pointer64<T>) -> Address {
        ptr.address.into()
    }
}

impl<T: ?Sized> From<Pointer64<T>> for u64 {
    #[inline(always)]
    fn from(ptr: Pointer64<T>) -> u64 {
        ptr.address
    }
}

/// Tries to convert a Pointer64 into a u32.
/// The function will return an `Error::Bounds` error if the input value is greater than `u32::max_value()`.
impl<T: ?Sized> TryFrom<Pointer64<T>> for u32 {
    type Error = crate::error::Error;

    fn try_from(ptr: Pointer64<T>) -> Result<u32, Self::Error> {
        if ptr.address <= (u32::max_value() as u64) {
            Ok(ptr.address as u32)
        } else {
            Err(crate::error::Error::Bounds)
        }
    }
}

// Arithmetic operations
impl<T> ops::Add<usize> for Pointer64<T> {
    type Output = Pointer64<T>;
    #[inline(always)]
    fn add(self, other: usize) -> Pointer64<T> {
        let address = self.address + (other * size_of::<T>()) as u64;
        Pointer64 {
            address,
            phantom_data: self.phantom_data,
        }
    }
}
impl<T> ops::Sub<usize> for Pointer64<T> {
    type Output = Pointer64<T>;
    #[inline(always)]
    fn sub(self, other: usize) -> Pointer64<T> {
        let address = self.address - (other * size_of::<T>()) as u64;
        Pointer64 {
            address,
            phantom_data: self.phantom_data,
        }
    }
}

impl<T: ?Sized> fmt::Debug for Pointer64<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl<T: ?Sized> fmt::UpperHex for Pointer64<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.address)
    }
}
impl<T: ?Sized> fmt::LowerHex for Pointer64<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl<T: ?Sized> fmt::Display for Pointer64<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}

unsafe impl<T: ?Sized + 'static> Pod for Pointer64<T> {}
const _: [(); std::mem::size_of::<Pointer64<()>>()] = [(); std::mem::size_of::<u64>()];

impl<T: ?Sized + 'static> ByteSwap for Pointer64<T> {
    fn byte_swap(&mut self) {
        self.address.byte_swap();
    }
}
