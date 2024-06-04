Within `demo` Docker container directory `/service/service_demo`.

File `.env.example` contains environemnt configuration parametters. At the
moment it contains all data and can just be copied to `.env` for ease of use,
in production we would not store usernames there.

To run development version, call:
```sh
cp .env.example .env
cargo run
```

To build release:
```sh
cargo build --release
```



# Project structure

`main.rs` - contains main function that sets up environment for threads. It is
very basic and expects to be managed by some process supervisor, like,
supervisord, docker, pm2, etc. If any thread crashes, main process crashes and
expects to be restarted.

`storage\postgres.rs` - implements async Storage trait, that is defined in
storage module. This demonstrates the use of impl in function arguments. And
writes accumulated data for 1 minute intervals.

`async_http_collector.rs` - this is the thread that creates requests to defined
HTTP endpoint, once per given period. It uses `rate_limit.rs` not to overwhelm
API endpoint. (In reality HTTP has so huge overhead that with normal network
connection in given case it is hard to exceed rate limit for given service)

`ohlc_calc.rs` - contains code that aggregates ticks into 1 minute
Open-High-Low-Close structures. This data is then propagated to terminal
(through AtomicSwap) and storage (through mpsc channel) threads.

`atomic_swap.rs` - implements basic functionality to swap boxed structs
atomically. This is used to demonstrate use of generics as well.

`terminal_output.rs` - implements thread that updates terminal output.

`bin\startup.sh` - small wrapper to conveniently start docker container. Here
we can change path to run development code instead of release without rebuilding
whole image.

Other files contain some basic shared structures between modules.

# Future improvements
1. Get rid of code that uses unwrap.
Since this is just a demo code, for prototyping i've used todo! macro and unwrap
calls at some places. For production code those cases should be propperly
handled without panicking.

2. Allow to define multiple crypto pairs. Since we internally use enum, enum
should be expanded as well.

3. Improve rate limiting capabilities. At the moment a rudimentary
implementation is made, but reusable code could be created that handles rate
limiting in a better way.

4. Allow to configure different verbosity levels for debugging purposes.

5. Currently we store 1 minute interval data into DB at aprox. 1 minute
intervals. This makes sense for current task at hand, but we might want to
batch multiple aggregated values and insert them in single INSERT call once per
10 minutes or so.

6. Some reusable code components could be moved to separate crates. This
repository could be refactored to use Cargo workspaces.
