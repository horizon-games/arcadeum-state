import * as arcadeum from 'arcadeum-bindings'
import * as ttt from 'arcadeum-ttt'
import * as ethers from 'ethers'

const Synchronous = false
;(async () => {
  const account = ethers.Wallet.createRandom()
  process.send(account.address)

  const store = {
    store: undefined as arcadeum.Store,
    unsubscribe: undefined as () => void
  }

  process.on(`message`, async (message: any) => {
    if (store.store === undefined) {
      const rootMessage = ethers.utils.arrayify(message)
      const send = (message: arcadeum.Message) =>
        process.send(ethers.utils.hexlify(message.encoding))
      store.store = new arcadeum.Store(ttt.Store, rootMessage, account, send)

      if (Synchronous) {
        await store.store.ready

        switch (store.store.player) {
          case arcadeum.Player.One:
            await store.store.opponentActions
            await store.store.dispatch([0, 0])
            await store.store.opponentActions
            await store.store.dispatch([0, 1])
            await store.store.opponentActions
            await store.store.dispatch([0, 2])

            console.log(
              `client (${process.pid}): ${ethers.utils.hexlify(
                await store.store.proof
              )}`
            )

            break

          case arcadeum.Player.Two:
            await store.store.opponentActions
            await store.store.dispatch([1, 0])
            await store.store.opponentActions
            await store.store.dispatch([1, 1])
            break
        }
      } else {
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
      }
    } else {
      store.store.receive(ethers.utils.arrayify(message))
    }
  })
})()
