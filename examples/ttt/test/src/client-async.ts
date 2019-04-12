import * as polkadot from '@polkadot/api'
import * as keyring from '@polkadot/keyring'
import * as arcadeum from 'arcadeum-bindings'
import * as ttt from 'arcadeum-ttt'
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
      ttt.Game,
      rootMessage,
      account,
      new Uint8Array(),
      (message: any) => {
        console.log(`client (${process.pid}):`)
        console.log(message)
      },
      send
    )

    store.unsubscribe = store.store.subscribe(() => {
      switch (store.store.nextPlayer) {
        case store.store.player:
          switch (store.store.sharedState.count) {
            case 0:
              store.store.dispatch([0, 0])
              break

            case 1:
              store.store.dispatch([1, 0])
              break

            case 2:
              store.store.dispatch([0, 1])
              break

            case 3:
              store.store.dispatch([1, 1])
              break

            case 4:
              store.store.dispatch([0, 2])
              break
          }

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
