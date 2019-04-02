import * as arcadeum from 'arcadeum-bindings'
import * as coin from 'arcadeum-coin'
import * as ethers from 'ethers'

const synchronous = false

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
    store.store = new arcadeum.Store(coin.Store, rootMessage, account, send)

    if (synchronous) {
      await store.store.ready

      while (store.store.nextPlayer !== undefined) {
        await store.store.opponentActions

        if (store.store.nextPlayer === store.store.player) {
          await store.store.dispatch(ethers.utils.randomBytes(1))
        }
      }

      if (store.store.winner === store.store.player) {
        console.log(
          `client (${process.pid}): ${ethers.utils.hexlify(
            await store.store.proof
          )}`
        )
      }
    } else {
      store.unsubscribe = store.store.subscribe(() => {
        switch (store.store.nextPlayer) {
          case store.store.player:
            store.store.dispatch(ethers.utils.randomBytes(1))
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
    }
  } else {
    store.store.receive(ethers.utils.arrayify(message))
  }
})
