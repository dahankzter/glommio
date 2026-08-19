//! A cell initialised at most once, possibly by an async initialiser.
//!
//! [`std::cell::OnceCell`] cannot help when the value must be produced by
//! something that awaits -- opening a file, resolving a name, asking a peer.
//! This one can, and it guarantees the initialiser runs once even when several
//! tasks reach the cell while initialisation is still in flight: the later
//! arrivals wait for the first rather than starting their own.
//!
//! # Examples
//!
//! ```
//! use glommio::{sync::OnceCell, LocalExecutor};
//!
//! let ex = LocalExecutor::default();
//! ex.run(async {
//!     let cell = OnceCell::new();
//!     let value = cell.get_or_init(|| async { 42 }).await;
//!     assert_eq!(*value, 42);
//! });
//! ```

use super::Semaphore;
use std::{cell::UnsafeCell, future::Future};

/// A cell holding a value that is initialised at most once.
#[derive(Debug)]
pub struct OnceCell<T> {
    value: UnsafeCell<Option<T>>,
    /// Held for the duration of an initialisation, so a second caller waits
    /// for the first instead of running its own initialiser.
    initialising: Semaphore,
}

impl<T> OnceCell<T> {
    /// Creates an empty cell.
    pub fn new() -> Self {
        OnceCell {
            value: UnsafeCell::new(None),
            initialising: Semaphore::new(1),
        }
    }

    /// Returns the value, or `None` if the cell is still empty.
    pub fn get(&self) -> Option<&T> {
        // Safety: the value is only ever written once, under the semaphore,
        // and is never removed or replaced afterwards -- so a shared reference
        // handed out here cannot be invalidated while it lives.
        unsafe { (*self.value.get()).as_ref() }
    }

    /// Returns whether the cell holds a value.
    pub fn is_initialized(&self) -> bool {
        self.get().is_some()
    }

    /// Sets the value if the cell is empty.
    ///
    /// # Errors
    ///
    /// Hands `value` back if the cell already holds one.
    pub fn set(&self, value: T) -> Result<(), T> {
        if self.is_initialized() {
            return Err(value);
        }

        // Safety: as in `get`, and nothing can be mid-initialisation here:
        // `set` does not await, so no other task can be running.
        unsafe { *self.value.get() = Some(value) };
        Ok(())
    }

    /// Returns the value, running `init` to produce it if the cell is empty.
    ///
    /// If another task is already initialising the cell, this waits for that
    /// one to finish rather than running `init` -- so the initialiser runs
    /// exactly once however many callers arrive.
    pub async fn get_or_init<F, Fut>(&self, init: F) -> &T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        if let Some(value) = self.get() {
            return value;
        }

        // Whoever takes the permit does the work; everyone else queues here
        // and finds the value already present when they get it.
        let _permit = self
            .initialising
            .acquire_permit(1)
            .await
            .expect("the initialisation semaphore is private and never closed");

        if self.get().is_none() {
            let value = init().await;
            // Safety: the permit makes this the only writer, and `get` above
            // proved nobody has published a value yet.
            unsafe { *self.value.get() = Some(value) };
        }

        self.get().expect("the cell was just initialised")
    }
}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        OnceCell::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{timer::Timer, LocalExecutor};
    use std::{cell::RefCell, rc::Rc, time::Duration};

    #[test]
    fn an_empty_cell_has_nothing_in_it() {
        let cell: OnceCell<u32> = OnceCell::new();
        assert!(cell.get().is_none());
        assert!(!cell.is_initialized());
    }

    #[test]
    fn set_stores_a_value_once() {
        let cell = OnceCell::new();
        assert!(cell.set(1).is_ok());
        assert_eq!(cell.get(), Some(&1));
        assert!(cell.is_initialized());
        assert_eq!(
            cell.set(2),
            Err(2),
            "a second set should hand the value back"
        );
        assert_eq!(cell.get(), Some(&1), "the first value must survive");
    }

    #[test]
    fn get_or_init_runs_the_initialiser_once() {
        LocalExecutor::default().run(async {
            let cell = OnceCell::new();
            let runs = Rc::new(RefCell::new(0));

            for _ in 0..3 {
                let value = cell
                    .get_or_init(|| {
                        let runs = runs.clone();
                        async move {
                            *runs.borrow_mut() += 1;
                            41
                        }
                    })
                    .await;
                assert_eq!(*value, 41);
            }

            assert_eq!(*runs.borrow(), 1, "the initialiser ran more than once");
        });
    }

    #[test]
    fn a_concurrent_get_or_init_waits_rather_than_initialising_again() {
        LocalExecutor::default().run(async {
            let cell = Rc::new(OnceCell::new());
            let runs = Rc::new(RefCell::new(0));

            // The first initialiser suspends, so the second arrives while
            // initialisation is still in flight.
            let slow = crate::spawn_local({
                let cell = cell.clone();
                let runs = runs.clone();
                async move {
                    *cell
                        .get_or_init(|| {
                            let runs = runs.clone();
                            async move {
                                Timer::new(Duration::from_millis(20)).await;
                                *runs.borrow_mut() += 1;
                                7
                            }
                        })
                        .await
                }
            })
            .detach();

            Timer::new(Duration::from_millis(5)).await;

            let second = *cell
                .get_or_init(|| {
                    let runs = runs.clone();
                    async move {
                        *runs.borrow_mut() += 1;
                        99
                    }
                })
                .await;

            assert_eq!(slow.await.unwrap(), 7);
            assert_eq!(second, 7, "the second caller should see the first value");
            assert_eq!(*runs.borrow(), 1, "the initialiser ran more than once");
        });
    }
}
