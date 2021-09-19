#![cfg(target_os = "linux")]

use std::{
    fs::File,
    io,
    marker::PhantomData,
    num::NonZeroUsize,
    ops::{BitOr, Deref, DerefMut},
    os::unix::{io::AsRawFd, prelude::MetadataExt},
};

use flag::{Flag, UniqueFlag};

mod flag;

fn mmap_anon(size: NonZeroUsize, prot: Protection) -> io::Result<*mut u8> {
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size.get(),
            prot.0,
            (UniqueFlag::MAP_SHARED | Flag::MAP_ANONYMOUS).0,
            -1,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        Err(io::Error::last_os_error())
    } else {
        Ok(ptr as *mut _)
    }
}

fn mmap_file(file: &File, prot: Protection) -> io::Result<(*mut u8, usize)> {
    let fd = file.as_raw_fd();

    let size = file.metadata()?.size() as usize;

    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            prot.0,
            UniqueFlag::MAP_SHARED.0,
            fd,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        Err(io::Error::last_os_error())
    } else {
        Ok((ptr as *mut _, size))
    }
}

pub struct Mmap<'a> {
    ptr: *const u8,
    len: usize,
    _lifetime: PhantomData<&'a ()>,
}

pub struct MmapMut<'a> {
    ptr: *mut u8,
    len: usize,
    _lifetime: PhantomData<&'a ()>,
}

macro_rules! mmap_impl {
    ($name:ident, $prot:ident, $target:ty) => {
        impl<'a> $name<'a> {
            pub fn new_anon(size: NonZeroUsize) -> io::Result<Self> {
                let ptr = mmap_anon(size, Protection::$prot)?;

                Ok(Self {
                    ptr,
                    len: size.get(),
                    _lifetime: PhantomData,
                })
            }

            pub fn new_anon_exec(size: NonZeroUsize) -> io::Result<Self> {
                let ptr = mmap_anon(size, Protection::$prot | Protection::EXEC)?;

                Ok(Self {
                    ptr,
                    len: size.get(),
                    _lifetime: PhantomData,
                })
            }

            pub fn new_file(file: &File) -> io::Result<Self> {
                let (ptr, len) = mmap_file(file, Protection::$prot)?;

                Ok(Self {
                    ptr,
                    len,
                    _lifetime: PhantomData,
                })
            }

            pub fn new_file_exec(file: &File) -> io::Result<Self> {
                let (ptr, len) = mmap_file(file, Protection::$prot | Protection::EXEC)?;

                Ok(Self {
                    ptr,
                    len,
                    _lifetime: PhantomData,
                })
            }
        }
    };
}

mmap_impl!(Mmap, READ, &'a [u8]);
mmap_impl!(MmapMut, WRITE, &'a mut [u8]);

impl<'a> Deref for Mmap<'a> {
    type Target = [u8];

    fn deref(&self) -> &'a Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'a> Deref for MmapMut<'a> {
    type Target = [u8];

    fn deref(&self) -> &'a Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'a> DerefMut for MmapMut<'a> {
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

#[repr(transparent)]
pub(crate) struct Protection(i32);

impl Protection {
    /// Pages may be read
    const READ: Self = Protection(libc::PROT_READ);

    /// Pages may be executed
    const EXEC: Self = Protection(libc::PROT_EXEC);

    /// Pages may be written
    const WRITE: Self = Protection(libc::PROT_WRITE);

    /// Pages may not be accessed
    #[allow(dead_code)]
    const NONE: Self = Protection(libc::PROT_NONE);
}

impl BitOr<Self> for Protection {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroUsize;

    use crate::{Mmap, MmapMut};

    #[test]
    fn anon_readonly() {
        let map = Mmap::new_anon(NonZeroUsize::new(20).unwrap()).unwrap();

        assert_eq!(&*map, &[0; 20]);
    }

    #[test]
    fn anon_mut() {
        let mut map = MmapMut::new_anon(NonZeroUsize::new(20).unwrap()).unwrap();

        assert_eq!(&*map, &[0; 20]);

        (&mut *map)[..].copy_from_slice(&[1; 20]);

        assert_eq!(&*map, &[1; 20]);
    }
}
