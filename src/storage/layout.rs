use std::fmt;
use std::ptr::NonNull;

/// Represents a specific side of the Order Book.
///
/// **Engineering Note:**
/// We use `#[repr(u8)]` to guarantee this enum takes exactly 1 byte.
/// This allows for efficient struct packing (filling alignment padding)
/// and enables branchless logic by casting to integer (0 or 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

/// A strongly-typed wrapper around `u64` to represent price.
///
/// **Zero-Cost Abstraction:**
/// `#[repr(transparent)]` guarantees this struct has the exact same
/// memory layout and ABI as a raw `u64`. This gives us compile-time
/// type safety (preventing Price + Quantity bugs) without paying a
/// performance penalty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Price(pub u64);

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display as fixed-point (5 decimal places)
        let s = format!("{:010}", self.0);
        let len = s.len();
        if len > 5 {
            write!(f, "{}.{}", &s[0..len - 5], &s[len - 5..])
        } else {
            write!(f, "0.{}", &s)
        }
    }
}

/// A strongly-typed wrapper around `u64` for quantity/size.
///
/// Uses `#[repr(transparent)]` to ensure identical layout to `u64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Quantity(pub u64);

impl Quantity {
    /// Helper to subtract safely, saturating at 0.
    pub fn saturating_sub(self, other: Self) -> Self {
        Quantity(self.0.saturating_sub(other.0))
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display as fixed-point (3 decimal places for volume)
        let s = format!("{:06}", self.0);
        let len = s.len();
        if len > 3 {
            write!(f, "{}.{}", &s[0..len - 3], &s[len - 3..])
        } else {
            write!(f, "0.{}", &s)
        }
    }
}

/// A unique identifier for an Order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct OrderId(pub u64);

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ID:{}", self.0)
    }
}

/// A pointer to an Order stored in the Arena.
///
/// **Engineering Decision: Why NonNull?**
/// We use `NonNull<Order>` instead of `Box` or `&mut` (or `Rc<RefCell>`) because:
///
/// 1.  **Covariance:** Unlike `*mut T` (which is invariant), `NonNull<T>` is covariant.
///     This means a pointer to a long-lived Order can be used where a short-lived
///     pointer is expected, simplifying lifetime management in our iterators.
/// 2.  **Optimization:** `Option<NonNull<T>>` is the same size as `*mut T` (64 bits).
///     Rust uses the "0" address to represent `None`, saving us from needing a
///     discriminant byte.
/// 3.  **Control:** Unlike `Rc<RefCell>`, this incurs **zero** runtime overhead for
///     access or mutation. Safety is enforced by the `OrderBook` structure (the Arena owner).
pub type OrderPtr = NonNull<Order>;

/// The Order Node stored in the Arena.
///
/// **Cache Line Analysis:**
/// - id (8) + price (8) + qty (8) + next (8) + prev (8) + side (1) = 41 bytes.
/// - Alignment padding (7 bytes) brings total size to 48 bytes.
/// - This fits comfortably within a standard 64-byte cache line.
#[derive(Debug, Clone)]
#[repr(C)] // Guarantees C-compatible field ordering
pub struct Order {
    pub id: OrderId,
    pub price: Price,
    pub qty: Quantity,

    // Intrusive Linked List Pointers
    pub next: Option<OrderPtr>,
    pub prev: Option<OrderPtr>,

    pub side: Side,
    // +7 bytes padding inserted by compiler here
}

impl Order {
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
