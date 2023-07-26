use std::panic;
use gio::{Cancellable, JoinHandle, Task};

// pub fn spawn_blocking<T, F>(func: F) -> JoinHandle<T>
//     where
//         T: Send + 'static,
//         F: FnOnce() -> T + Send + 'static,
// {
//     // use Cancellable::NONE as source obj to fulfill `Send` requirement
//     // let task = unsafe { Task::<bool>::new(Cancellable::NONE, Cancellable::NONE, |_, _| {}) };
//     // let (join, tx) = JoinHandle::new();
//     // task.run_in_thread(move |_, _: Option<&Cancellable>, _| {
//     //     let res = panic::catch_unwind(panic::AssertUnwindSafe(func));
//     //     let _ = tx.send(res);
//     // });
//     //
//     // join
// }