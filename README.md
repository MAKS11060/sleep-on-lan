<h1 align="center">Sleep-on-Lan</h1>
<h3 align="center">Switches the PC to sleep mode using WoL <a href="https://wikipedia.org/wiki/Wake-on-LAN">(Wake-on-LAN)</a></h3>

<!-- <div align="center"> -->
<!-- [![CI](https://github.com/MAKS11060/sleep-on-lan/actions/workflows/ci.yml/badge.svg)](https://github.com/MAKS11060/sleep-on-lan/actions/workflows/ci.yml) -->
<!-- </div> -->

<div align="center">
  <a href="https://github.com/MAKS11060/sleep-on-lan/actions/workflows/ci.yml">
    <img src="https://github.com/MAKS11060/sleep-on-lan/actions/workflows/ci.yml/badge.svg">
  </a>
</div>

## Install

1. Install service
```ps
cargo run --bin service install
# or remove
cargo run --bin service uninstall
```

## Todo:
- [x] Rewrite on rust
- [x] Add `windows service`
- [ ] Add installer
- [ ] Write doc
