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
    store.store = new arcadeum.Store(ttt.Store, rootMessage, account, send)

    await store.store.ready

    switch (store.store.player) {
      case arcadeum.Player.One:
        await store.store.opponentActions
        await store.store.dispatch([0, 0])
        await store.store.opponentActions
        await store.store.dispatch([0, 1])
        await store.store.opponentActions
        await store.store.dispatch([0, 2])

        if (store.store.winner === store.store.player) {
          console.log(
            `client (${process.pid}): ${ethers.utils.hexlify(
              await store.store.proof
            )}`
          )
        }

        break

      case arcadeum.Player.Two:
        await store.store.opponentActions
        await store.store.dispatch([1, 0])
        await store.store.opponentActions
        await store.store.dispatch([1, 1])
        break
    }
  } else {
    store.store.receive(ethers.utils.arrayify(message))
  }
})
