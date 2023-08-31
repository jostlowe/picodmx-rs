# [WIP] picodmx-rs

## Description

This library enables the rpi pico to communicate with the dmx interface in rust.
This library currently only supports dmx output

## Quickstart

### Hardware

The rpi pico needs to be wired via a [Logic Level Converter](https://www.amazon.com/logic-level-converter/s?k=logic+level+converter) to a RS-485 signal chain (the dmx cable in this case)
via a converter like the [MAX485](https://www.amazon.com/s?k=MAX485&crid=2HHO41UOFM15&sprefix=max48%2Caps%2C220&ref=nb_sb_noss_2).

### Software

Uppon setting up the normal chain of rust code to get up and running on the pico, the dmx interface needs to be initialized.

```rust
Dmx::new(&mut pio, sm, dmx_pin_id, &clocks.system_clock).unwrap();
```

In your loop you just have to make one call to send your dmx frame.

```rust
dmx.send_blocking(0x00, &[0xff, 0xaa, 0xff, 0xaa]);
```

There are some calculations on what would be the fastest rate at which dmx can be send, but to be safe, call this about every 52ms.
The [example](https://github.com/jostlowe/picodmx-rs/blob/master/examples/output.rs) just sets a delay in the loop.

