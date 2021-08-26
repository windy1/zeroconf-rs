# zeroconf

`zeroconf` is a cross-platform library that wraps underlying [ZeroConf/mDNS] implementations
such as [Bonjour] or [Avahi], providing an easy and idiomatic way to both register and
browse services.

## Prerequisites

On Linux:

```bash
$ sudo apt install xorg-dev libxcb-shape0-dev libxcb-xfixes0-dev clang avahi-daemon libavahi-client-dev
```

## TODO

* Windows support
* You tell me...

## Examples

Please refer to [the docs] for examples.

## Resources

* [Avahi docs]
* [Bonjour docs]

[ZeroConf/mDNS]: https://en.wikipedia.org/wiki/Zero-configuration_networking
[Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)
[Avahi]: https://en.wikipedia.org/wiki/Avahi_(software)
[`Any`]: https://doc.rust-lang.org/std/any/trait.Any.html
[Avahi docs]: https://avahi.org/doxygen/html/
[Bonjour docs]: https://developer.apple.com/documentation/dnssd/dns_service_discovery_c
[the docs]: https://docs.rs/zeroconf
