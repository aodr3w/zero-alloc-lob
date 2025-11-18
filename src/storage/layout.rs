use std::num::NonZeroU32;

///Represents a specific side of the Order Book.
///
/// **Engineering note**
/// We use `#[repr(u8)]` to guarantee this enum takes exactly 1 byte.
/// This is critical for struct packing if we ever embed this in a larger struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

/// A strongly-typed wrapper around `u64` to represent price.
///
/// *Why not f64?**
/// Floating point math is non-deterministic across different CPU architectures
/// and prone to precision errors (e.g ,0.1 + 0.2 != 0.3).
/// In financial engineering, we always use fixed-point arithmetic (integers).
///
/// e.g price 50,000,000 might be stored as 5,000,000,000 (with 1e5 scale)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)] // Guarantees layout is identicial to u64
pub struct Price(pub u64);

/// A strongly-typed wrapper around `u64` for quantity/size
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Quantity(pub u64);

/// A unique identifier for an Order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct OrderId(pub u64);

/// A handle (index) to an Order stored in the Arena
///
/// ** Engineering Decision: **
/// We use `u32` instead of `usize` (64-bit) to save memory.
/// This limits us to ~4 billion active orders, which is sufficient for any single matching engine.
///
/// `NonZeroU32` allows Rust to use the "0" value as `None`.
/// This means `Option<OrderIndex>` is still 4 bytes, not 8 bytes (Discriminant Optimization)
pub type OrderIndex = NonZeroU32;

/// The Order Node stored in the Arena.
///
/// This struct is designed to be "Cache Line Friendly".
/// A standard CPU cache line is 64 bytes.
///
/// **Layout Analysis:**
/// - id: 8 bytes
/// - price: 8 bytes
/// - qty: 8 bytes
/// - next: 4 bytes
/// - prev: 4 bytes
/// - side: 1 byte
/// - _padding: 31 bytes (reserved for future flags, timestamps, or alignment)
///
/// Total size should ideally align to 64 bytes to prevent "False Sharing" if accessed across threads,
/// though strictly 64-byte alignment is more critical for the *start* of the allocation.
#[derive(Debug, Clone)]
#[repr(C)] // Standard C layout for predictable padding
pub struct Order {
    /// The unique ID of the order (8 bytes)
    pub id: OrderId,

    /// The limit price (8 bytes)
    pub price: Price,

    /// The open quantity (8 bytes)
    pub qty: Quantity,

    /// Intrusive Linked List: Pointer to the next order in the queue (4 bytes)
    pub next: Option<OrderIndex>,

    /// Intrusive Linked List: Pointer to the previous order in the queue (4 bytes)
    pub prev: Option<OrderIndex>,

    /// Buy or Sell (1 byte)
    pub side: Side,
    // Explicit padding isn't strictly required as Rust/LLVM handles alignment,
    // but in HFT we often inspect this manually.
    // Current Size: 8+8+8+4+4+1 = 33 bytes.
    // This will be padded to 40 bytes (alignment of u64).
}

impl Order {
    /// Creates a new order with no links.
    pub fn new(id: OrderId, side: Side, price: Price, qty: Quantity) -> Self {
        Self {
            id,
            side,
            price,
            qty,
            next: None,
            prev: None,
        }
    }
}

/*
additional engineering commentary
NonZeroU32 vs usize: I used u32 for the OrderIndex. On a 64-bit machine, a standard pointer (Box<Order>) is 8 bytes. By using an index (u32), we cut the "pointer" size in half.
This effectively doubles the number of relationships we can store in the CPU cache.
#[repr(transparent)]: This is a "zero-cost abstraction." It tells the compiler, "Treat Price exactly like a u64 in memory, but don't let me accidentally add a Price to a Quantity in code."
Intrusive Linked List: Notice next and prev are inside the Order struct. In standard scripting (Python/JS), you might have a List object containing Order objects. In Systems Engineering, the Order knows its place in the list. This removes the need for a separate "List Node" allocation.

*/
