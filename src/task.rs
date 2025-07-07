/*
A task represents a unit of work that can be executed by the executor.
It contains a future that is being executed and a spawner that is used to send the task
to the executor's ready queue.
A waker is a handle that allows a task to be woken up when it is ready to make progress.
It is used by the executor to notify tasks that they can be polled again.
This handle encapsulates the RawWaker, which define the executor specific behavior for waking up tasks.
*/

use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Waker, RawWaker, RawWakerVTable}};

use crate::executor::Spawner;

pub(crate) struct Task {
    pub future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
    pub spawner: Spawner,
}

impl Task {
    const WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    pub fn waker(self: Arc<Self>) -> Waker {
        let opaque_ptr = Arc::into_raw(self) as *const ();
        let vtable = &Self::WAKER_VTABLE;

        unsafe { Waker::from_raw(RawWaker::new(opaque_ptr, vtable)) }
    }
}

fn clone(ptr: *const ()) -> RawWaker {
    let original: Arc<Task> = unsafe { Arc::from_raw(ptr as _) };

    // Increment the inner counter of the arc.
    let cloned = original.clone();

    // now forget the Arc<Task> so the refcount isn't decremented
    std::mem::forget(original);
    std::mem::forget(cloned);

    RawWaker::new(ptr, &Task::WAKER_VTABLE)
}

fn drop(ptr: *const ()) {
    let _: Arc<Task> = unsafe { Arc::from_raw(ptr as _) };
}

fn wake(ptr: *const ()) {
    let arc: Arc<Task> = unsafe { Arc::from_raw(ptr as _) };
    let spawner = arc.spawner.clone();

    spawner.spawn_task(arc);
}

fn wake_by_ref(ptr: *const ()) {
    let arc: Arc<Task> = unsafe { Arc::from_raw(ptr as _) };

    arc.spawner.spawn_task(arc.clone());

    // we don't actually have ownership of this arc value
    // therefore we must not drop `arc`
    std::mem::forget(arc)
}
