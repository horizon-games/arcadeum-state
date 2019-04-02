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
    store.store = new arcadeum.Store(ttt.Game, rootMessage, account, send)

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
          if (store.store.winner === store.store.player) {
            store.store.proof.then((proof: Uint8Array) => {
              console.log(
                `client (${process.pid}): ${ethers.utils.hexlify(proof)}`
              )
            })
          }

          store.unsubscribe()
          break
      }
    })
  } else {
    store.store.receive(ethers.utils.arrayify(message))
  }
})
