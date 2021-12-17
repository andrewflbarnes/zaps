# ZAPS: A Payments Simulator

See what I did with the name ;)

For learning some Rust fundamentals inc. networking, multi-threading, futures, etc.

Inspired by an 8583 simulator I wrote at a previous place in Java so I have a good idea of what needs to be done even if I'm
not sure yet how I'll accomplish it!

## Running

Start the server with
```bash
cargo r
```

It is hardcoded to listen on port 9090.

Connections can be established from multiple clients e.g. using
```bash
nc localhost: 9090
```

Messages are echoed to all clients (except the sender).

To use the ISO8583 engine prefix a message with `is8583:` and send a valid payload matching the spec from `zaps-sim` i.e.
```
MTI:            0200
Primary Bitmap: 8 bits ASCII packed hex (2 bytes)
1:              LLLVar alpha
8:              Fixed 15 alphanumeric
```

For example:
```
iso8583:020081003ABC0123456789abcde
```

The tokenised message (still early days!) will be echoed to other clients e.g.
```
{1: "ABC", 8: "0123456789abcde"}
```
