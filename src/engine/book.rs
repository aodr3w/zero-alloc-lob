use crate::engine::matcher::{self, Trade};
use crate::storage::layout::{Order, OrderId, OrderPtr, Price, Quantity, Side};
use llt_rs::arena_allocator::Arena;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

pub struct OrderBook {
    symbol: &'static str,
    order_arena: Arena,

    free_list: Vec<OrderPtr>,

    order_index: HashMap<OrderId, OrderPtr>,

    pub(crate) best_bid: Option<OrderPtr>,
    pub(crate) best_ask: Option<OrderPtr>,

    _marker: PhantomData<Order>,
}

impl OrderBook {
    pub fn new(symbol: &'static str, capacity: usize) -> Self {
        let order_size = mem::size_of::<Order>();
        let total_bytes = capacity * order_size;

        Self {
            symbol,
            order_arena: Arena::new(total_bytes),
            // Reserve space for the pointers so 'push' never allocates
            free_list: Vec::with_capacity(capacity),
            order_index: HashMap::with_capacity(capacity),
            best_bid: None,
            best_ask: None,
            _marker: PhantomData,
        }
    }

    pub fn symbol(&self) -> &'static str {
        self.symbol
    }

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

        if self.order_index.contains_key(&order_id) {
            return Err(format!("Duplicate Order ID: {}", order_id.0));
        }

        // --- STEP 1: MATCHING (TAKER) ---
        let (remaining_qty, trades) = matcher::execute_match(self, order_id, side, price, qty);

        if remaining_qty.0 == 0 {
            return Ok((None, trades));
        }

        // --- STEP 2: PLACEMENT (MAKER) ---
        let new_order_data = Order::new(order_id, side, price, remaining_qty);

        // ALLOCATION STRATEGY:
        // 1. Check the Free List (O(1) Pop)
        // 2. If empty, Bump Allocate from Arena (O(1) Pointer bump)
        let order_ptr = if let Some(mut recycled_ptr) = self.free_list.pop() {
            // RECYCLING: We are writing new data into an "old" memory address
            unsafe {
                *recycled_ptr.as_mut() = new_order_data;
            }
            recycled_ptr
        } else {
            // ALLOCATION: New memory from the big block
            let order_ref = self.order_arena.alloc(new_order_data);
            unsafe { NonNull::new_unchecked(order_ref as *mut Order) }
        };

        // INSERTION (O(N) - Price-Time Priority)
        unsafe {
            self.insert_sorted(order_ptr, side, price);
        }

        // INDEXING (CONTROL PLANE)
        self.order_index.insert(order_id, order_ptr);

        Ok((Some(order_ptr), trades))
    }

    /// Modifies an existing order.
    pub fn modify_order(
        &mut self,
        id: u64,
        new_price: u64,
        new_qty: u64,
    ) -> Result<(Option<OrderPtr>, Vec<Trade>), String> {
        let order_id = OrderId(id);
        let new_price = Price(new_price);
        let new_qty = Quantity(new_qty);

        let mut order_ptr = match self.order_index.get(&order_id) {
            Some(ptr) => *ptr,
            None => return Err(format!("Order ID {} not found.", id)),
        };

        // Safety: We hold mutable reference to book
        let order = unsafe { order_ptr.as_mut() };

        // FAST PATH: Price match + Qty reduction
        if order.price == new_price && new_qty <= order.qty {
            order.qty = new_qty;
            if new_qty.0 == 0 {
                self.cancel_order(id)?;
                return Ok((None, vec![]));
            }
            return Ok((Some(order_ptr), vec![]));
        }

        // SLOW PATH: Price change or Qty increase -> Loss of Priority
        let side = order.side;
        self.cancel_order(id)?;
        self.place_limit_order(id, side, new_price.0, new_qty.0)
    }

    pub fn cancel_order(&mut self, id: u64) -> Result<OrderId, String> {
        let order_id = OrderId(id);

        let order_ptr = match self.order_index.remove(&order_id) {
            Some(ptr) => ptr,
            None => return Err(format!("Order ID {} not found in book.", id)),
        };

        // 1. O(1) Unlink
        self.remove_order(order_ptr);

        // 2. O(1) Recycle: Push the pointer onto the free list stack
        self.free_list.push(order_ptr);

        Ok(order_id)
    }

    pub(crate) fn remove_order(&mut self, mut ptr: OrderPtr) {
        unsafe {
            let order = ptr.as_mut();
            let next_ptr = order.next;
            let prev_ptr = order.prev;

            if let Some(mut next) = next_ptr {
                next.as_mut().prev = prev_ptr;
            }

            if let Some(mut prev) = prev_ptr {
                prev.as_mut().next = next_ptr;
            }

            if self.best_bid == Some(ptr) {
                self.best_bid = next_ptr;
            }
            if self.best_ask == Some(ptr) {
                self.best_ask = next_ptr;
            }

            order.next = None;
            order.prev = None;
        }
    }

    unsafe fn insert_sorted(&mut self, mut new_ptr: OrderPtr, side: Side, price: Price) {
        let mut current_ptr = match side {
            Side::Buy => self.best_bid,
            Side::Sell => self.best_ask,
        };

        let mut prev_ptr: Option<OrderPtr> = None;

        while let Some(curr) = current_ptr {
            let curr_order = unsafe { curr.as_ref() };

            let should_insert_before = match side {
                Side::Buy => price > curr_order.price,
                Side::Sell => price < curr_order.price,
            };

            if should_insert_before {
                break;
            }
            prev_ptr = Some(curr);
            current_ptr = curr_order.next;
        }

        unsafe {
            new_ptr.as_mut().next = current_ptr;
        }
        unsafe {
            new_ptr.as_mut().prev = prev_ptr;
        }

        if let Some(mut curr) = current_ptr {
            unsafe {
                curr.as_mut().prev = Some(new_ptr);
            }
        }
        if let Some(mut prev) = prev_ptr {
            unsafe {
                prev.as_mut().next = Some(new_ptr);
            }
        } else {
            match side {
                Side::Buy => self.best_bid = Some(new_ptr),
                Side::Sell => self.best_ask = Some(new_ptr),
            }
        }
    }

    pub fn best_ask_price(&self) -> Option<Price> {
        self.best_ask.map(|ptr| unsafe { ptr.as_ref().price })
    }

    pub fn best_bid_price(&self) -> Option<Price> {
        self.best_bid.map(|ptr| unsafe { ptr.as_ref().price })
    }

    pub fn capacity_bytes(&self) -> usize {
        self.order_arena.capacity()
    }

    pub fn used_bytes(&self) -> usize {
        self.order_arena.used_bytes()
    }

    pub fn active_orders(&self) -> usize {
        self.order_index.len()
    }

    pub fn free_slots(&self) -> usize {
        self.free_list.len()
    }
}
