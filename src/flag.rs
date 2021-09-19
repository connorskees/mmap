use std::ops::{BitOr, Deref};

/// Only one of these flags may be present
pub(crate) struct UniqueFlag(pub(crate) i32);

#[allow(dead_code)]
impl UniqueFlag {
    /// Share this mapping. Updates to the mapping are visible to other processes
    /// mapping the same region, and (in the case of file-backed mappings) are
    /// carried through to the underlying file. (To precisely control when updates
    /// are carried through to the underlying file requires the use of msync(2).)
    pub(crate) const MAP_SHARED: Self = Self(libc::MAP_SHARED);

    /// This flag provides the same behavior as MAP_SHARED except that MAP_SHARED
    /// mappings ignore unknown flags in flags. By contrast, when creating a
    /// mapping using MAP_SHARED_VALIDATE, the kernel verifies all passed flags
    /// are known and fails the mapping with the error EOPNOTSUPP for unknown
    /// flags. This mapping type is also required to be able to use some mapping
    /// flags (e.g., MAP_SYNC).
    ///
    /// (since Linux 4.15)
    pub(crate) const MAP_SHARED_VALIDATE: Self = Self(libc::MAP_SHARED_VALIDATE);

    /// Create a private copy-on-write mapping. Updates to the mapping are not
    /// visible to other processes mapping the same file, and are not carried
    /// through to the underlying file. It is unspecified whether changes made
    /// to the file after the mmap() call are visible in the mapped region.
    pub(crate) const MAP_PRIVATE: Self = Self(libc::MAP_PRIVATE);
}

pub(crate) struct Flag(i32);

#[allow(dead_code)]
impl Flag {
    /// Put the mapping into the first 2 Gigabytes of the process address space.
    /// This flag is supported only on x86-64, for 64-bit programs. It was added
    /// to allow thread stacks to be allocated somewhere in the first 2 GB of
    /// memory, so as to improve context-switch performance on some early 64-bit
    /// processors. Modern x86-64 processors no longer have this performance problem,
    /// so use of this flag is not required on those systems. The MAP_32BIT flag
    /// is ignored when MAP_FIXED is set.
    ///
    /// (since Linux 2.4.20, 2.6)
    pub(crate) const MAP_32BIT: Self = Self(libc::MAP_32BIT);

    /// The mapping is not backed by any file; its contents are initialized to
    /// zero. The fd argument is ignored; however, some implementations require
    /// fd to be -1 if MAP_ANONYMOUS (or MAP_ANON) is specified, and portable
    /// applications should ensure this. The offset argument should be zero. The
    /// use of MAP_ANONYMOUS in conjunction with MAP_SHARED is supported on Linux
    /// only since kernel 2.4.
    pub(crate) const MAP_ANONYMOUS: Self = Self(libc::MAP_ANONYMOUS);

    /// Don't interpret addr as a hint: place the mapping at exactly that address.
    /// addr must be suitably aligned: for most architectures a multiple of the
    /// page size is sufficient; however, some architectures may impose additional
    /// restrictions. If the memory region specified by addr and length overlaps
    /// pages of any existing mapping(s), then the overlapped part of the existing
    /// mapping(s) will be discarded. If the specified address cannot be used,
    /// mmap() will fail.
    pub(crate) const MAP_FIXED: Self = Self(libc::MAP_FIXED);

    /// This flag provides behavior that is similar to MAP_FIXED with respect to
    /// the addr enforcement, but differs in that MAP_FIXED_NOREPLACE never clobbers
    /// a preexisting mapped range. If the requested range would collide with an
    /// existing mapping, then this call fails with the error EEXIST. This flag
    /// can therefore be used as a way to atomically (with respect to other threads)
    /// attempt to map an address range: one thread will succeed; all others will
    /// report failure.
    ///
    /// Note that older kernels which do not recognize the MAP_FIXED_NOREPLACE
    /// flag will typically (upon detecting a collision with a preexisting mapping)
    /// fall back to a "non- MAP_FIXED" type of behavior: they will return an
    /// address that is different from the requested address. Therefore,
    /// backward-compatible software should check the returned address against
    /// the requested address.
    ///
    /// (since Linux 4.17)
    pub(crate) const MAP_FIXED_NOREPLACE: Self = Self(libc::MAP_FIXED_NOREPLACE);

    /// This flag is used for stacks. It indicates to the kernel virtual memory
    /// system that the mapping should extend downward in memory. The return
    /// address is one page lower than the memory area that is actually created
    /// in the process's virtual address space. Touching an address in the "guard"
    /// page below the mapping will cause the mapping to grow by a page. This
    /// growth can be repeated until the mapping grows to within a page of the
    /// high end of the next lower mapping, at which point touching the "guard"
    /// page will result in a SIGSEGV signal.
    pub(crate) const MAP_GROWSDOWN: Self = Self(libc::MAP_GROWSDOWN);

    /// Allocate the mapping using "huge" pages. See the Linux kernel source file
    /// Documentation/admin-guide/mm/hugetlbpage.rst for further information, as
    /// well as NOTES, below.
    ///
    /// (since Linux 2.6.32)
    pub(crate) const MAP_HUGETLB: Self = Self(libc::MAP_HUGETLB);

    /// Used in conjunction with MAP_HUGETLB to select alternative hugetlb page
    /// sizes (respectively, 2 MB and 1 GB) on systems that support multiple
    /// hugetlb page sizes.
    ///
    /// More generally, the desired huge page size can be configured by encoding
    /// the base-2 logarithm of the desired page size in the six bits at the
    /// offset MAP_HUGE_SHIFT. (A value of zero in this bit field provides the
    /// default huge page size; the default huge page size can be discovered via
    /// the Hugepagesize field exposed by /proc/meminfo.) Thus, the above two
    /// pub(crate) constants are defined as:
    ///
    ///    #define MAP_HUGE_2MB   (21 << MAP_HUGE_SHIFT)
    ///    #define MAP_HUGE_1GB   (30 << MAP_HUGE_SHIFT)
    ///
    /// The range of huge page sizes that are supported by the system can be
    /// discovered by listing the subdirectories in /sys/kernel/mm/hugepages.
    ///
    /// (since Linux 3.8)
    pub(crate) const MAP_HUGE_2MB: Self = Self(libc::MAP_HUGE_2MB);
    pub(crate) const MAP_HUGE_1GB: Self = Self(libc::MAP_HUGE_1GB);

    /// Mark the mapped region to be locked in the same way as mlock(2). This
    /// implementation will try to populate (prefault) the whole range but the
    /// mmap() call doesn't fail with ENOMEM if this fails. Therefore major faults
    /// might happen later on. So the semantic is not as strong as mlock(2). One
    /// should use mmap() plus mlock(2) when major faults are not acceptable after
    /// the initialization of the mapping. The MAP_LOCKED flag is ignored in older
    /// kernels.
    ///
    /// (since Linux 2.5.37)
    pub(crate) const MAP_LOCKED: Self = Self(libc::MAP_LOCKED);

    /// This flag is meaningful only in conjunction with MAP_POPULATE. Don't
    /// perform read-ahead: create page tables entries only for pages that are
    /// already present in RAM. Since Linux 2.6.23, this flag causes MAP_POPULATE
    /// to do nothing. One day, the combination of MAP_POPULATE and MAP_NONBLOCK
    /// may be reimplemented.
    ///
    /// (since Linux 2.5.46)
    pub(crate) const MAP_NONBLOCK: Self = Self(libc::MAP_NONBLOCK);

    /// Do not reserve swap space for this mapping. When swap space is reserved,
    /// one has the guarantee that it is possible to modify the mapping. When
    /// swap space is not reserved one might get SIGSEGV upon a write if no physical
    /// memory is available. See also the discussion of the file
    /// /proc/sys/vm/overcommit_memory in proc(5). In kernels before 2.6, this
    /// flag had effect only for private writable mappings.
    pub(crate) const MAP_NORESERVE: Self = Self(libc::MAP_NORESERVE);

    /// Populate (prefault) page tables for a mapping. For a file mapping, this
    /// causes read-ahead on the file. This will help to reduce blocking on page
    /// faults later. The mmap() call doesn't fail if the mapping cannot be
    /// populated (for example, due to limitations on the number of mapped huge
    /// pages when using MAP_HUGETLB). MAP_POPULATE is supported for private
    /// mappings only since Linux 2.6.23.
    ///
    /// (since Linux 2.5.46)
    pub(crate) const MAP_POPULATE: Self = Self(libc::MAP_POPULATE);

    /// Allocate the mapping at an address suitable for a process or thread stack.
    ///
    /// This flag is currently a no-op on Linux. However, by employing this flag,
    /// applications can ensure that they transparently obtain support if the
    /// flag is implemented in the future. Thus, it is used in the glibc threading
    /// implementation to allow for the fact that some architectures may (later)
    /// require special treatment for stack allocations. A further reason to
    /// employ this flag is portability: MAP_STACK exists (and has an effect)
    /// on some other systems (e.g., some of the BSDs).
    ///
    /// (since Linux 2.6.27)
    pub(crate) const MAP_STACK: Self = Self(libc::MAP_STACK);

    /// This flag is available only with the MAP_SHARED_VALIDATE mapping type;
    /// mappings of type MAP_SHARED will silently ignore this flag. This flag is
    /// supported only for files supporting DAX (direct mapping of persistent
    /// memory). For other files, creating a mapping with this flag results in
    /// an EOPNOTSUPP error.
    ///
    /// Shared file mappings with this flag provide the guarantee that while
    /// some memory is mapped writable in the address space of the process, it
    /// will be visible in the same file at the same offset even after the system
    /// crashes or is rebooted. In conjunction with the use of appropriate CPU
    /// instructions, this provides users of such mappings with a more efficient
    /// way of making data modifications persistent.
    ///
    /// (since Linux 4.15)
    pub(crate) const MAP_SYNC: Self = Self(libc::MAP_SYNC);
}

impl Deref for Flag {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BitOr<Self> for Flag {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOr<UniqueFlag> for Flag {
    type Output = UniqueFlag;
    fn bitor(self, rhs: UniqueFlag) -> Self::Output {
        UniqueFlag(self.0 | rhs.0)
    }
}

impl BitOr<Flag> for UniqueFlag {
    type Output = UniqueFlag;
    fn bitor(self, rhs: Flag) -> Self::Output {
        UniqueFlag(self.0 | rhs.0)
    }
}
