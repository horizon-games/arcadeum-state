/*
 * Arcadeum blockchain game framework
 * Copyright (C) 2019  Horizon Blockchain Games Inc.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 3.0 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

import * as polkadot from '@polkadot/api'
import * as keyring from '@polkadot/keyring'
import * as arcadeum from 'arcadeum-bindings'
import * as rps from 'arcadeum-rps'
import * as ethers from 'ethers'

const account = ethers.Wallet.createRandom()
process.send(account.address)

const store = {
  store: undefined as arcadeum.Store | undefined,
  unsubscribe: undefined as (() => void) | undefined
}

process.on(`message`, async (message: any) => {
  if (store.store === undefined) {
    const rootMessage = ethers.utils.arrayify(message)
    const send = (message: arcadeum.Message) =>
      process.send(ethers.utils.hexlify(message.encoding))

    store.store = new arcadeum.Store(
      rps.Game,
      rootMessage,
      account,
      new Uint8Array(),
      (message: any) => {
        console.log(`client (${process.pid}): ${JSON.stringify(message)}`)
      },
      send
    )

    store.unsubscribe = store.store.subscribe(() => {
      switch (store.store.nextPlayer) {
        case store.store.player:
          let random = ethers.utils.randomBytes(1)[0]

          while (random === 255) {
            random = ethers.utils.randomBytes(1)[0]
          }

          store.store.dispatch([random % 3])
          break

        case undefined:
          switch (store.store.winner) {
            case store.store.player:
              console.log(`client (${process.pid}): ${account.address} won`)

              store.store.proof.then(async (proof: Uint8Array) => {
                const provider = new polkadot.WsProvider(`ws://localhost:9944`)
                const api = await polkadot.ApiPromise.create(provider)
                const keys = new keyring.Keyring({ type: `sr25519` })
                const key = keys.addFromUri(`//Alice`)

                api.tx.arcadeum
                  .prove(ethers.utils.hexlify(proof))
                  .signAndSend(key)
              })

              break

            case undefined:
              console.log(`client (${process.pid}): ${account.address} drew`)
              break

            default:
              console.log(`client (${process.pid}): ${account.address} lost`)
              break
          }

          store.unsubscribe()
          break
      }
    })
  } else {
    store.store.receive(ethers.utils.arrayify(message))
  }
})