use crate::storage::layout::{Order, OrderId, OrderPtr, Price, Quantity, Side};
use llt_rs::arena_allocator::Arena;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

pub struct OrderBook {
    symbol: &'static str,
    order_arena: Arena, // The untyped byte-buffer arena
    best_bid: Option<OrderPtr>,
    best_ask: Option<OrderPtr>,
    _marker: PhantomData<Order>,
}

impl OrderBook {
    /*
    Initialize a new OrderBook with pre-allocated memory.
    # Arguments
    * `symbol` - Trading pair name.
    * `capacity` - The number of ORDERS to pre-allocate space for.
    */
    pub fn new(symbol: &'static str, capacity: usize) -> Self {
        // 1. Calculate total bytes needed.
        let order_size = mem::size_of::<Order>();
        let total_bytes = capacity * order_size;
        Self {
            symbol,
            order_arena: Arena::new(total_bytes),
            best_bid: None,
            best_ask: None,
            _marker: PhantomData,
        }
    }
    /// The Hot Path: Placing an order.
    pub fn place_limit_order(
        &mut self,
        id: u64,
        side: Side,
        price: u64,
        qty: u64,
    ) -> Result<Option<OrderPtr>, String> {
        let price = Price(price);
        let qty = Quantity(qty);
        let order_id = OrderId(id);

        //1 . Create the struct on the stack.
        let new_order_data = Order::new(order_id, side, price, qty);

        //2 Allocate in Arena ( moves data to heap)
        // SAFETY: The Arena returns a mutable reference with a lifetime tied to &self.
        // Since OrderBook owns the Arena, we cant keep that reference safely
        // We convert it to a raw pointer immediately
        let order_ref = self.order_arena.alloc(new_order_data);

        //Convert &nut Order -> NonNull<Order>
        let mut order_ptr = unsafe { NonNull::new_unchecked(order_ref as *mut Order) };

        //3. Link into the book (LIFO / Stack behaviour for this example )
        // In a real engine, this would be a sorted insert (WE NEED TO DO THIS)

        match side {
            Side::Buy => {
                unsafe {
                    // Set current head as next
                    order_ptr.as_mut().next = self.best_bid;
                    //if there was a head, update its prev
                    if let Some(mut head) = self.best_bid {
                        head.as_mut().prev = Some(order_ptr);
                    }
                    // Update Book Head
                    self.best_bid = Some(order_ptr);
                }
            }
            Side::Sell => unsafe {
                order_ptr.as_mut().next = self.best_ask;
                if let Some(mut head) = self.best_ask {
                    head.as_mut().prev = Some(order_ptr);
                }
                self.best_ask = Some(order_ptr);
            },
        }
        Ok(Some(order_ptr))
    }

    pub fn capacity_bytes(&self) -> usize {
        self.order_arena.capacity()
    }

    pub fn used_bytes(&self) -> usize {
        self.order_arena.used_bytes()
    }
}
