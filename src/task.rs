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
