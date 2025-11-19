use crate::engine::matcher::{self, Trade};
use crate::storage::layout::{Order, OrderId, OrderPtr, Price, Quantity, Side};
use llt_rs::arena_allocator::Arena;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

pub struct OrderBook {
    symbol: &'static str,
    order_arena: Arena,

    pub(crate) best_bid: Option<OrderPtr>,
    pub(crate) best_ask: Option<OrderPtr>,

    // PhantomData to correctly signal ownership of Orders to the drop checker
    _marker: PhantomData<Order>,
}

impl OrderBook {
    pub fn new(symbol: &'static str, capacity: usize) -> Self {
        let order_size = mem::size_of::<Order>();
        // Memory-perfect allocation: capacity * size
        let total_bytes = capacity * order_size;

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
        // If we are here, the order is now a "Resting Order".
        let new_order_data = Order::new(order_id, side, price, remaining_qty);

        // ALLOCATION (Hot Path - Unsafe due to arena interaction)
        let order_ref = self.order_arena.alloc(new_order_data);
        let order_ptr = unsafe { NonNull::new_unchecked(order_ref as *mut Order) };

        unsafe {
            self.insert_sorted(order_ptr, side, price);
        }

        Ok((Some(order_ptr), trades))
    }

    /// Removes an order from the linked list.
    /// Crucial for when an order is fully filled during matching.
    pub(crate) fn remove_order(&mut self, mut ptr: OrderPtr) {
        unsafe {
            // Safety: We assume ptr is valid and points to memory owned by the arena.
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

            // Clear pointers on the removed order
            order.next = None;
            order.prev = None;
        }
    }

    /// Inserts a new order into the linked list maintaining Price-Time priority.
    /// O(N) operation in this implementation (Linear Scan)
    unsafe fn insert_sorted(&mut self, mut new_ptr: OrderPtr, side: Side, price: Price) {
        let mut current_ptr = match side {
            Side::Buy => self.best_bid,
            Side::Sell => self.best_ask,
        };

        let mut prev_ptr: Option<OrderPtr> = None;

        // TRAVERSAL: Find the insertion point
        while let Some(curr) = current_ptr {
            // Safety: Accessing Order data through pointer, must be wrapped.
            let curr_order = unsafe { curr.as_ref() };

            // Stop if we find a spot where the new order belongs BEFORE the current one.
            let should_insert_before = match side {
                // Buy Side (Descending): Insert if New Price > Current Price.
                Side::Buy => price > curr_order.price,
                // Sell Side (Ascending): Insert if New Price < Current Price.
                Side::Sell => price < curr_order.price,
            };

            if should_insert_before {
                break;
            }

            // Advance the pointers
            prev_ptr = Some(curr);
            current_ptr = curr_order.next;
        }

        // INSERTION: We are inserting `new_ptr` between `prev_ptr` and `current_ptr`.

        // 1. Link New -> Next (Current)
        unsafe {
            new_ptr.as_mut().next = current_ptr;
        }

        // 2. Link New -> Prev (Prev)
        unsafe {
            new_ptr.as_mut().prev = prev_ptr;
        }

        // 3. Link Next -> New
        if let Some(mut curr) = current_ptr {
            unsafe {
                curr.as_mut().prev = Some(new_ptr);
            }
        }

        // 4. Link Prev -> New OR Update Head
        if let Some(mut prev) = prev_ptr {
            unsafe {
                prev.as_mut().next = Some(new_ptr);
            }
        } else {
            // If no prev, we are the new Head
            match side {
                Side::Buy => self.best_bid = Some(new_ptr),
                Side::Sell => self.best_ask = Some(new_ptr),
            }
        }
    }

    pub fn capacity_bytes(&self) -> usize {
        self.order_arena.capacity()
    }

    pub fn used_bytes(&self) -> usize {
        self.order_arena.used_bytes()
    }
}
