# TMT (Too Many Tabs)

I always have too many tabs open. I wanted a place to put them, so I made one.

## About

_Extremely_ basic CRUD app to store links & notes. Probably don't use this unless
you're literally me.

### Requirements/Installation/Use

- Rust >= 1.6.6
  - Probably? I'm not consciously doing anything particularly cutting-edge but
    who even knows what dark magic goes into `axum`.
  - Definitely `edition = "2021"` at least.
- SQLite3 >= 3???
  - I've got uhhh `3.37.2` so that, I guess? Again I'm not doing anything wild
    with it.

I wrote and ran this on a System76 Galago Pro 2 laptop running Ubuntu 22.04. It
_should_ probably work on other platforms but I make no promises.

### Installation etc

- `git clone https://github.com/swizzard/tmt`
- `mkdir ~/.tmt && cd ~/.tmt && ln -s <where you cloned tmt>/templates`
- `cargo run` (or `cargo install --path .`)

It's currently hardcoded to run on port `9999`, so visit `http://localhost:9999`
and you should be off to the races.

## To Do

- [ ] Literally any styling--at the very least some `viewport` futzing to make it
      mobile-friendly
- [ ] Configuration?? The path to the sqlite3 db and the port it runs on are hard-coded,
      which is fine for Literally Just Me but might be worth making configurable
      so anyone else can use it
- [ ] Packaging it up as a crate and/or executable(s)
  - [ ] The templates are all pretty small so I'm sure I could embed them in the
        binary, somehow.
