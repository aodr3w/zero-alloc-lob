use std::ptr::NonNull;

/// Represents a specific side of the Order Book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

/// A strongly-typed wrapper around `u64` to represent price.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Price(pub u64);

/// A strongly-typed wrapper around `u64` for quantity/size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Quantity(pub u64);

/// A unique identifier for an Order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct OrderId(pub u64);

/// A pointer to an Order stored in the Arena.
///
/// **Engineering Change:**
/// We switched from `u32` indices to `NonNull<Order>`.
/// Since `llt-rs::Arena` returns `&mut T` (pointers), we must store pointers.
/// `NonNull` is covarient and optimized (Option<NonNull> is same size as *mut T).
pub type OrderPtr = NonNull<Order>;

/// The Order Node stored in the Arena.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Order {
    /// The unique ID of the order (8 bytes)
    pub id: OrderId,

    /// The limit price (8 bytes)
    pub price: Price,

    /// The open quantity (8 bytes)
    pub qty: Quantity,

    /// Intrusive Linked List: Pointer to the next order (8 bytes)
    pub next: Option<OrderPtr>,

    /// Intrusive Linked List: Pointer to the previous order (8 bytes)
    pub prev: Option<OrderPtr>,

    /// Buy or Sell (1 byte)
    pub side: Side,
    // Padding to align to 8 bytes (Total size ~48 bytes)
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
