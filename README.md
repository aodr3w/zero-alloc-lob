# zero-alloc-lob

**A deterministic, zero-allocation Limit Order Book(LOB) engine written in Rust.**

**⚠ Status**: Active Development / Private Alpha


**🎯 Goal**: < 500ns matching latency (tick-to-trade)


## 📖 Overview

**zero-alloc-lob** is a high-performance matching engine designed for environments where tail latency matters more than average latency (e.g., HFT, Market Making, and Exchange Infrastructure).

Unlike standard matching engines that rely on dynamic memory allocation (causing non-deterministic heap fragmentation and GC/allocator pauses), this engine operates entirely on pre-allocated memory structures. It guarantees O(1) or amortized O(1) memory operations on the hot path.

## ⚡ Key Features

**Zero Dynamic Allocation**: All order objects and nodes are managed via static Arenas and Object Pools provided by llt-rs. No malloc/free calls during the trading session.

**Cache Locality**: Optimized for L1/L2 cache hits by using contiguous memory layouts rather than pointer chasing.

**Deterministic Execution**: The engine state is purely a function of the input sequence, making it ideal for replay-based debugging and high-fidelity backtesting.

**Safe Concurrency**: Built on Rust's safety guarantees, ensuring memory safety without the overhead of garbage collection.


# 🏗 Architecture

The engine is built on top of the llt-rs (Low Latency Toolkit) ecosystem.

## Core Dependencies

**llt-rs**: Provides the underlying memory primitives.

feature = ["arena_allocator"]: Used for storing Order Nodes (Red-Black Tree or Splay Tree nodes).

feature = ["object-pool"]: Used for recycling Order objects to minimize construction overhead.

## Data Layout

**Bids/Asks**: Stored in cache-friendly binary trees (or flat maps for dense price levels).

**Order Storage**: Intrusive linked lists backed by slab allocation to ensure stable memory addresses without pointers.


## 🚀 Performance Targets

Metric                Target          Current

**Add Order**         < 200 ms         TBD

**Cancel Order**      < 150ns          TBD

**Match Execution**   < 500ns          TBD

**Jitter (99th %ile)**  < 1µs          TBD


## 📦 Installation & Usage

This library is currently private. It relies on the local llt-rs crate.



## 🛠 Roadmap

* [ ] Phase 1: Memory Infrastructure

    * Integration with llt-rs Arena.

    * Definition of OrderNode and OrderBook structs.

* [ ] Phase 2: Matching Logic
    * FIFO (Price-Time) matching algorithm.
    * Partial fills and aggressive matching.

* [ ] Phase 3: Order Management
    * Cancel, Replace/Modify support.

* [ ] Phase 4: Integration
    * Python bindings (via PyO3) for backtesting.
    * Simple TCP/UDP gateway.


## PROJECT LAYOUT

```
zero-alloc-lob/
├── Cargo.toml
├── README.md
├── benches/              # Criterion benchmarks (CRITICAL for this project)
│   └── latency.rs
├── src/
│   ├── lib.rs            # Public API
│   ├── engine/           # The matching logic
│   │   ├── mod.rs
│   │   ├── book.rs       # The Book struct
│   │   └── matcher.rs    # The execution logic
│   └── storage/          # The memory layout
│       ├── mod.rs
│       └── layout.rs     # Where we use llt-rs arenas
└── examples/
    └── simple_match.rs   # A runnable example

```
