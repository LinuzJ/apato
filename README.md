# apato

WIP: Rental apartment yield watchlist service. Written in Rust.

Calculates the yields of apartments in regions included in the watchlists specified by the user.

Technologies used:

- Rust
- Diesel
- Postgresql

## Usage

Run db:

```
docker-compose up -d
```

Run service:

```
cargo run main.rs
```

Run migrations:

```
diesel migration run
```

## Bot commands

Subscribe to a watchlist at location `id` and set the wanted yield to be `yield`

```
   /sub {location id} {yield}
```

Unsubscribe to a watchlist with watchlist id `id`

```
   /unsub {watchlist id}
```

Lists all the current watchlists of the caller

```
   /listsubs
```

Get information about all apartments currently in `watchlist id`

```
   /getall {watchlist id}
```

Get information about all apartments in the wanted yield range currently in `watchlist id`

```
   /getallvalid
```

Helper for all commands

```
   /help
```
