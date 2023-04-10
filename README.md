# uip

A tool to quickly determine the public IP address of a machine behind
NAT with the help of STUN servers.

# Overview

This tool is as simple as it sounds, when executed, it outputs the current machine's
public IP address by sending a binding request to the STUN server and displaying the
`XOR-MAPPED-ADDRESS` or `MAPPED-ADDRESS` attribute returned by the STUN server.

# Install

Either download pre-built binaries from the release page, or use `cargo install uip` to install it.

# Usage

```
$ uip -h
A tool to quickly determine the public IP address of a machine behind
NAT with the help of STUN servers.


Usage: uip [HOST:PORT]

Arguments:
  [HOST:PORT]  STUN server host and port. If PORT is omitted, it defaults to 3478 [default: stun.l.google.com:19302]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Default port (UDP 3478):

```
$ uip stun.l.google.com
203.0.113.1
```

## Custom port:

```
$ uip stun.l.google.com:19302
203.0.113.1
```

This program use return code `0` to indicate success and a valid IP has been returned.
All other return code indicates error.

The program timeouts after `5` seconds in case a response can not be received.

To debug, run it with `RUST_LOG=debug uip`.

# Credits

This script was modified from the [`stun_client.rs`](https://github.com/webrtc-rs/webrtc/blob/master/stun/examples/stun_client.rs)
example of the [WebRTC.rs project](https://github.com/webrtc-rs/webrtc) by Rain Liu <yliu@webrtc.rs>.

`uip` added better error handling, more friendly command line interface and fallback to `MAPPED-ADDRESS` attribute
in case `XOR-MAPPED-ADDRESS` can not be found since some servers that I tested does not return ``XOR-MAPPED-ADDRESS`.

# License

Copyright 2023 Datong Sun (dndx@idndx.com)

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
[https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0)> or the MIT license
<LICENSE-MIT or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT)>, at your
option. Files in the project may not be
copied, modified, or distributed except according to those terms.
