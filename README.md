A minimal async runtime and reactor in Rust, built by following [this excellent tutorial from Tweedegolf](https://tweedegolf.nl/en/blog/114/building-an-async-runtime-with-mio).

Runs a basic UDP echo server, demonstrating how to manage async tasks and socket events manually, from first principles (Rust + mio).

---

## 📝 How This Works (Big Picture)

![image](https://github.com/user-attachments/assets/61ecc5cc-6dd6-4ca3-87e6-5f7671c614e8)

```text
┌──────────────────────────────────────────────┐
│ 1. main()                                    │
│    - Creates executor and spawner            │
│    - Spawns async_main() as a task           │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 2. Executor loop begins:                     │
│    - Takes task from queue                   │
│    - Polls task's future                     │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 3. Task runs async_main()                    │
│    - Calls recv_from().await on socket       │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 4. recv_from() tries reading socket          │
│    - If data ready: returns immediately      │
│    - If WouldBlock: needs to wait!           │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 5. poll_fn future is awaited:                │
│    - Executor calls closure with Context (cx)│
│    - Closure calls Reactor::poll(token, cx)  │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 6. Reactor::poll():                          │
│    - Stores task's waker in map under token  │
│    - Returns Poll::Pending                   │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 7. Executor sees Pending:                    │
│    - Task is paused (not re-polled)          │
│    - Executor runs other tasks or waits      │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 8. Reactor thread waits for OS events        │
│    - OS signals socket is ready (event)      │
│    - Reactor loop wakes up                   │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 9. Reactor handles event:                    │
│    - Updates map: Status::Happened           │
│    - If Status::Awaited(waker) existed:      │
│        - Calls waker.wake()                  │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 10. waker.wake() puts task back on executor  │
│     - Task is ready to be polled again       │
└─────────┬────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────────────┐
│ 11. Executor polls task again                │
│     - Task resumes at .await point           │
│     - recv_from() now succeeds               │
│     - async_main() continues or completes    │
└──────────────────────────────────────────────┘
