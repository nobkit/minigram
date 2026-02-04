<h1 align="center"> <picture><source srcset="./assets/miniroute-red.svg"><img alt="miniroute logo" height="48" ></picture> <br> Miniroute </h1>


<p align="center">
A minimalistic router for embedded Rust projects written with <a href="https://github.com/embassy-rs/embassy">Embassy</a>.
</p>

## ✨ Features

- 📦 **Task grouping:** Organize related Embassy tasks into routes that start and stop as a unit.
- 🔄 **Managed lifecycle:** Both routes and individual tasks have optional **`setup`** and **`cleanup`** phases. Route hooks bracket the group; task hooks nest inside.
- 🤝 **Graceful navigation:** Route switches are cooperative. Tasks yield at natural `.await` boundaries and acknowledge shutdown before the next route activates. Peripheral access and interrupt handlers are never forcibly interrupted.
- 🪶 **Lightweight:** All state is statically allocated. No `std` and no allocator required.

## ⚙️ Installation

You must have a working Embassy setup in order to use *Miniroute* meaningfully - see their [Getting Started](https://embassy.dev/book/#_getting_started) section.

After ensuring that the executor is running, you can run the following command to add *Miniroute*:

```bash
cargo add miniroute --features defmt
```

> [!NOTE]
> The `defmt` feature is optional, but it is the only logging library available with *Miniroute* as of now. Excluding this feature will compile each logging invocation into a no-op.

## 👀 Example

A router with two routes (`Home` and `Temperature`). The `Temperature` route runs a sensor task and an input task. When the user presses the home button, the input task navigates back to `Home`.

### 📄 `main.rs`
```rust
#[router]
enum Route {
    #[to(Home)]
    Home,
    #[to(Temperature)]
    Temperature,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    Route::spawn_routes(&spawner);
    /* Start on the temperature route */
    Route::start(Route::Temperature);
}
```
### 📄 `routes/temperature.rs`
```rust
#[route(router = Route, hooks)]
enum Temperature {
    #[task(sensor)]
    Sensor,
    #[task(input)]
    Input,
}

impl RouteHooks for Temperature {
    async fn setup() { /* ... */ }
    async fn cleanup() { /* ... */ }
}

#[embassy_executor::task]
async fn sensor(route: Temperature) {
    route
        .task(async || { /* ... */ })
        .setup(async || { /* ... */ })
        .cleanup(async || { /* ... */ })
        .run().await;
}

#[embassy_executor::task]
async fn input(route: Temperature) {
  route
      .task(async || {
        /* An example of using navigate() when an event is captured */
        match INPUT_RECEIVER.receive().await {
          InputEvent::HomeButton => route.navigate(Route::Home)
          _ => { /* ... */ }
        };
      })
      .run().await;
}
```
