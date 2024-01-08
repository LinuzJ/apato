# apato

WIP: Rental apartment yield watchlist service. Written in Rust.

Calculates the yields of apartments in regions included in the watchlists specified by the user.

Technologies used:

- Rust
- Rocket
- Diesel
- Postgresql

## Usage

Run db:

```
docker-compose up -d
```

Run server:

```
cargo watch -x run
```

Run migrations:

```
diesel migration run
```
