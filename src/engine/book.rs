use crate::engine::matcher::{self, Trade};
use crate::storage::layout::{Order, OrderId, OrderPtr, Price, Quantity, Side};
use llt_rs::arena_allocator::Arena;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

pub struct OrderBook {
    symbol: &'static str,
    order_arena: Arena,

    // Exposed to crate for matcher.rs
    pub(crate) best_bid: Option<OrderPtr>,
    pub(crate) best_ask: Option<OrderPtr>,

    // PhantomData to correctly signal ownership of Orders to the drop checker
    _marker: PhantomData<Order>,
}

impl OrderBook {
    pub fn new(symbol: &'static str, capacity: usize) -> Self {
        let order_size = mem::size_of::<Order>();
        // Capacity bytes + padding buffer
        let total_bytes = capacity * order_size + (capacity * 64);

        Self {
            symbol,
            order_arena: Arena::new(total_bytes),
            best_bid: None,
            best_ask: None,
            _marker: PhantomData,
        }
    }

    pub fn symbol(&self) -> &'static str {
        self.symbol
    }

    /// Places a limit order.
    ///
    /// This function now:
    /// 1. Matches against existing orders (Taking liquidity).
    /// 2. If quantity remains, inserts into the book (Making liquidity).
    pub fn place_limit_order(
        &mut self,
        id: u64,
        side: Side,
        price: u64,
        qty: u64,
    ) -> Result<(Option<OrderPtr>, Vec<Trade>), String> {
        let price = Price(price);
        let qty = Quantity(qty);
        let order_id = OrderId(id);

        // --- STEP 1: MATCHING (TAKER) ---
        let (remaining_qty, trades) = matcher::execute_match(self, order_id, side, price, qty);

        // If fully filled, we are done.
        if remaining_qty.0 == 0 {
            return Ok((None, trades));
        }

        // --- STEP 2: PLACEMENT (MAKER) ---
        // If we are here, it means the order is now a "Resting Order".
        // We allocate it in the Arena and link it into the list.

        let new_order_data = Order::new(order_id, side, price, remaining_qty);

        // ALLOCATION (Hot Path)
        let order_ref = self.order_arena.alloc(new_order_data);
        let mut order_ptr = unsafe { NonNull::new_unchecked(order_ref as *mut Order) };

        // INSERTION
        // Note: A real LOB requires Sorted Insert (O(N) or O(log N)).
        // For this Phase 1 prototype, we are doing Head Insert (Stack Behavior).
        // This is incorrect for price-time priority but validates the memory model.
        match side {
            Side::Buy => unsafe {
                order_ptr.as_mut().next = self.best_bid;
                if let Some(mut head) = self.best_bid {
                    head.as_mut().prev = Some(order_ptr);
                }
                self.best_bid = Some(order_ptr);
            },
            Side::Sell => unsafe {
                order_ptr.as_mut().next = self.best_ask;
                if let Some(mut head) = self.best_ask {
                    head.as_mut().prev = Some(order_ptr);
                }
                self.best_ask = Some(order_ptr);
            },
        }

        Ok((Some(order_ptr), trades))
    }

    /// Removes an order from the linked list.
    /// Crucial for when an order is fully filled during matching.
    pub(crate) fn remove_order(&mut self, mut ptr: OrderPtr) {
        unsafe {
            let order = ptr.as_mut();
            let next_ptr = order.next;
            let prev_ptr = order.prev;

            // 1. Unlink Next
            if let Some(mut next) = next_ptr {
                next.as_mut().prev = prev_ptr;
            }

            // 2. Unlink Prev
            if let Some(mut prev) = prev_ptr {
                prev.as_mut().next = next_ptr;
            }

            // 3. Update Head Pointers if necessary
            if self.best_bid == Some(ptr) {
                self.best_bid = next_ptr;
            }
            if self.best_ask == Some(ptr) {
                self.best_ask = next_ptr;
            }

            // Clear pointers on the removed order to be safe
            order.next = None;
            order.prev = None;

            // Note: We do NOT free the memory in the Arena.
            // In a bump allocator, memory is only reclaimed on `reset()`.
            // This creates "holes" (fragmentation), which is why we need
            // the Object Pool (Phase 3) to recycle these slots later.
        }
    }

    pub fn capacity_bytes(&self) -> usize {
        self.order_arena.capacity()
    }

    pub fn used_bytes(&self) -> usize {
        self.order_arena.used_bytes()
    }
}
