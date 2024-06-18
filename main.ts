#!/usr/bin/env -S deno run -A --watch

import {debounce} from 'jsr:@std/async'
import { suspend } from "./sys-suspend.ts";

/* WOL Pocket (102 bytes)
1. ffffffffffff // Constant

2. 102030405060 // Target MAC
3. 102030405060
4. 102030405060
5. 102030405060
6. 102030405060
7. 102030405060
8. 102030405060
9. 102030405060

1. 102030405060
2. 102030405060
3. 102030405060
4. 102030405060
5. 102030405060
6. 102030405060
7. 102030405060
8. 102030405060
*/

// Get MAC addresses of network interfaces
const macList = Deno.networkInterfaces()
  .filter((int) => int.family === 'IPv4')
  .map((int) => {
    const mac = Uint8Array.from(int.mac.split(':').map((v) => parseInt(v, 16)))
    return {mac}
  })

const listenWOL = (port: number = 9) => {
  const listener = Deno.listenDatagram({
    port,
    transport: 'udp',
    reuseAddress: true,
    hostname: '0.0.0.0',
  })

  if (listener.addr.transport === 'udp') {
    console.log(`Listen ${listener.addr.hostname}:${listener.addr.port} (UDP)`)
  }

  return new ReadableStream<{
    isWoL: boolean
    currentDevice: boolean
    addr: Deno.Addr
  }>({
    async pull(c) {
      for await (const [data, addr] of listener) {
        if (data.byteLength === 102) {
          const isWoL = data.subarray(0, 6).every((v) => v === 255)
          if (!isWoL) continue

          let currentDevice = true
          for (let i = 6; i < data.byteLength; i += 6) {
            currentDevice = !!macList.find(({mac}) => {
              return data.slice(i, i + 6).every((v, i) => v === mac.at(i))
            })
          }

          c.enqueue({isWoL, currentDevice, addr})
        }
      }
    },
    cancel() {
      listener.close()
    },
  })
}

// for await (const data of listenWOL()) {
//   console.log(data)
//   break
// }

const debounceTime = 1000 * 60 // * 3

let last = 0

Array.fromAsync(
  listenWOL(),
  debounce((data) => {
    if (data.currentDevice) {
      if (Date.now() - last >= debounceTime) {
        last = Date.now() // set now
        console.log('fire')
        suspend(false, true, false)
      }
    }
  }, 200)
)
