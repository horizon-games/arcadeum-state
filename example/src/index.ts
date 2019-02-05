import * as ethers from 'ethers'
import * as bindings from 'bindings'

;(async () => {
  try {
    const matcher = ethers.Wallet.fromMnemonic(`winter off snap small sleep debate cheap drill elevator glove caution once`)
    const account1 = ethers.Wallet.fromMnemonic(`scatter seed defense relief purity agree visit budget actress punch invest help`)
    const account2 = ethers.Wallet.fromMnemonic(`album nominee rate flavor rather trip inflict treat snap argue steak lawsuit`)
    const subkey1 = ethers.Wallet.createRandom()
    const subkey2 = ethers.Wallet.createRandom()

    console.log(`matcher: ${matcher.address}`)
    console.log(`player 1: ${account1.address}`)
    console.log(`player 2: ${account2.address}`)

    let state = new bindings.State()

    let message = await bindings.createRootMessage(new Uint8Array(16), account1.address, account2.address, matcher)
    state = state.next(message)
    let proof = message.encoding.slice()

    message = await bindings.createSubkeyMessage(subkey1.address, account1, message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await bindings.createSubkeyMessage(subkey2.address, account2, message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    let action = await bindings.createActionMessage([0, 0], subkey1, message)
    message = action[0]
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await bindings.createReplyMessage(subkey2, message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await action[1](message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    action = await bindings.createActionMessage([1, 0], subkey2, message)
    message = action[0]
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await bindings.createReplyMessage(subkey1, message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await action[1](message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    action = await bindings.createActionMessage([0, 1], subkey1, message)
    message = action[0]
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await bindings.createReplyMessage(subkey2, message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await action[1](message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    action = await bindings.createActionMessage([1, 1], subkey2, message)
    message = action[0]
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await bindings.createReplyMessage(subkey1, message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await action[1](message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    action = await bindings.createActionMessage([0, 2], subkey1, message)
    message = action[0]
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await bindings.createReplyMessage(subkey2, message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    message = await action[1](message)
    state = state.next(message)
    proof = concat(proof, message.encoding)

    console.log(`proof: ${ethers.utils.hexlify(proof)}`)

  } catch (error) {
    console.log(error)
  }
})()

function concat(a: Uint8Array, b: Uint8Array): Uint8Array {
  const c = new Uint8Array(a.length + b.length)

  c.set(a)
  c.set(b, a.length)

  return c
}
