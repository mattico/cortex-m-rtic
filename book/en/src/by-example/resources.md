# Resources

The framework provides an abstraction to share data between any of the contexts
we saw in the previous section (task handlers, `init` and `idle`): resources.

Resources are data visible only to functions declared within the `#[app]`
module. The framework gives the user complete control over which context
can access which resource.

All resources are declared as a single `struct` within the `#[app]`
module. Each field in the structure corresponds to a different resource.
The `struct` must be annotated with the following attribute: `#[resources]`.

Resources can optionally be given an initial value using the `#[init]`
attribute. Resources that are not given an initial value are referred to as
*late* resources and are covered in more detail in a follow-up section in this
page.

Each context (task handler, `init` or `idle`) must declare the resources it
intends to access in its corresponding metadata attribute using the `resources`
argument. This argument takes a list of resource names as its value. The listed
resources are made available to the context under the `resources` field of the
`Context` structure.

The example application shown below contains two interrupt handlers that share access to a resource named `shared`.

``` rust
{{#include ../../../../examples/resource.rs}}
```

``` console
$ cargo run --example resource
{{#include ../../../../ci/expected/resource.run}}
```

Note that the `shared` resource cannot be accessed from `idle`. Attempting to do so results in a compile error.

## `lock`

Critical sections are required to access shared mutable data in a data race-free manner.

The `resources` field of the passed `Context` implements the [`Mutex`] trait for each shared resource accessible to the task.

The only method on this trait, [`lock`], runs its closure argument in a critical section.

[`Mutex`]: ../../../api/rtic/trait.Mutex.html
[`lock`]: ../../../api/rtic/trait.Mutex.html#method.lock

The critical section created by the `lock` API is based on dynamic priorities: it temporarily raises the dynamic priority of the context to a *ceiling* priority that prevents other tasks from preempting the critical section. This synchronization protocol is known as the [Immediate Ceiling Priority Protocol
(ICPP)][icpp], and complies with [Stack Resource Policy(SRP)][srp] based scheduling of RTIC.

[icpp]: https://en.wikipedia.org/wiki/Priority_ceiling_protocol
[srp]: https://en.wikipedia.org/wiki/Stack_Resource_Policy

In the example below we have three interrupt handlers with priorities ranging from one to three. The two handlers with the lower priorities contend for the `shared` resource and need to lock the resource for accessing the data. The highest priority handler, which do nat access the `shared` resource, is free to preempt the critical section created by the
lowest priority handler.

``` rust
{{#include ../../../../examples/lock.rs}}
```

``` console
$ cargo run --example lock
{{#include ../../../../ci/expected/lock.run}}
```

## Multi-lock

As an extension to `lock`, and to reduce rightward drift, locks can be taken as tuples. The following examples shows this in use:

``` rust
{{#include ../../../../examples/multilock.rs}}
```

## Late resources

Late resources are resources that are not given an initial value at compile time using the `#[init]` attribute but instead are initialized at runtime using the `init::LateResources` values returned by the `init` function.

Late resources are useful e.g., to *move* (as in transferring the ownership of) peripherals initialized in `init` into tasks.

The example below uses late resources to establish a lockless, one-way channel between the `UART0` interrupt handler and the `idle` task. A single producer single consumer [`Queue`] is used as the channel. The queue is split into consumer and producer end points in `init` and then each end point is stored in a different resource; `UART0` owns the producer resource and `idle` owns the consumer resource.

[`Queue`]: ../../../api/heapless/spsc/struct.Queue.html

``` rust
{{#include ../../../../examples/late.rs}}
```

``` console
$ cargo run --example late
{{#include ../../../../ci/expected/late.run}}
```

## Only shared access

By default the framework assumes that all tasks require exclusive access (`&mut-`) to resources but it is possible to specify that a task only requires shared access (`&-`) to a resource using the `&resource_name` syntax in the `resources` list.

The advantage of specifying shared access (`&-`) to a resource is that no locks are required to access the resource even if the resource is contended by several tasks running at different priorities. The downside is that the task only gets a shared reference (`&-`) to the resource, limiting the operations it can perform on it, but where a shared reference is enough this approach reduces the number of required locks. In addition to simple immutable data, this shared access can be useful where the resource type safely implements interior mutability, with
appropriate locking or atomic operations of its own.

Note that in this release of RTIC it is not possible to request both exclusive access (`&mut-`) and shared access (`&-`) to the *same* resource from different tasks. Attempting to do so will result in a compile error.

In the example below a key (e.g. a cryptographic key) is loaded (or created) at runtime and then used from two tasks that run at different priorities without any kind of lock.

``` rust
{{#include ../../../../examples/only-shared-access.rs}}
```

``` console
$ cargo run --example only-shared-access
{{#include ../../../../ci/expected/only-shared-access.run}}
```

## Lock-free resource access of mutable resources

There exists two other options dealing with resources

* `#[lock_free]`: there might be several tasks with the same priority
  accessing the resource without critical section. Since tasks with the
  same priority never can preempt another task on the same priority
  this is safe.
* `#[task_local]`: there must be only one task using this resource,
  similar to a `static mut` task local resource, but (optionally) set-up by init.

