# Preface

This book is a guided tour of **Moss**, a Unix-like operating system kernel written in Rust. Moss is designed to be readable, correct, and educational — making it an ideal vehicle for learning how modern operating systems work from the inside out.

## Who This Book Is For

This book is intended for students taking an introductory operating systems course who want to see the concepts they are learning in a real, runnable kernel. It assumes:

- Comfort with at least one systems programming language (C, C++, or Rust)
- Familiarity with basic computer architecture concepts (registers, memory, stack)
- Some exposure to command-line tools

You do **not** need to be a Rust expert. Where Rust-specific features appear, they are explained in terms of the operating-systems concept they enforce.

## What You Will Learn

By the end of this book you will understand how a kernel:

1. Starts from bare hardware with no operating system underneath it
2. Manages physical and virtual memory, including demand paging and copy-on-write
3. Creates, schedules, and destroys processes
4. Handles hardware interrupts and software exceptions
5. Provides a clean interface between user programs and hardware via system calls
6. Abstracts storage behind a virtual filesystem layer
7. Delivers signals between processes

Every chapter ties theory directly to source code in the Moss repository, so you can read, run, and modify what you are learning.

## How to Read This Book

The book is organized into seven parts:

- **Part I** lays the conceptual foundations and walks through the boot sequence
- **Parts II and III** cover the two pillars of every kernel: memory management and process management
- **Part IV** examines how the kernel interacts with hardware via system calls and interrupts
- **Part V** covers I/O, storage, and device drivers
- **Part VI** explains how processes communicate via signals
- **Part VII** is a practical guide to building, running, and testing Moss

Each chapter ends with exercises ranging from "read the source" questions to small programming tasks that extend the kernel.

## A Note on Rust

Moss is written in Rust, a language that enforces memory safety at compile time. This is not just an aesthetic choice — many of the safety properties we care about in a kernel (no use-after-free, no data races, no invalid pointer dereferences) are properties the Rust compiler verifies for us. Where kernel code *must* bypass the type system (e.g., to set up page tables or read hardware registers), it uses `unsafe` blocks that are carefully documented and audited.

Throughout the book, Rust syntax is explained as it appears. If you want a broader introduction to the language, [The Rust Book](https://doc.rust-lang.org/book/) is an excellent companion.

## Source Code Conventions

Code references use the notation `path/to/file.rs:line_number`. For example:

> The task structure is defined at `src/process/mod.rs:42`.

All paths are relative to the root of the Moss repository at `https://github.com/moss-kernel/moss-kernel`.

Let's begin.
