# Chapter 8: The Virtual Filesystem

Files are the universal abstraction in Unix. The **Virtual Filesystem (VFS)** is the kernel layer that provides a uniform interface to all storage: hard drives, RAM-backed filesystems, network filesystems, kernel-generated data (procfs), and special device files.

From user space, reading from a file on an NVMe disk looks the same as reading from a tmpfs file or from `/proc/cmdline`. The VFS makes this possible.

## Learning Objectives

By the end of this chapter you should be able to:

- Describe the VFS design and its key abstractions (inodes, open files, mounts)
- Explain path resolution, including symbolic links and mount points
- List the filesystem drivers included in Moss
- Understand the purpose of special filesystems (procfs, devfs, tmpfs)

## Contents

- [VFS Design](./design.md)
- [Inodes and Open Files](./inodes.md)
- [Path Resolution](./path-resolution.md)
- [Filesystem Drivers](./drivers.md)
- [Special Filesystems](./special-fs.md)
