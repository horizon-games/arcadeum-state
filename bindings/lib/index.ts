import * as ethers from 'ethers'
import * as bindings from './bindings'

const Matcher = `0x373513E36c78044A08A35D237C94Ec49F362e372`

export class State {
  get decoding(): any {
    if (this.state !== undefined) {
      return this.state.decode()
    }
  }

  get winner(): Player | undefined {
    if (this.state !== undefined) {
      return this.state.winner()
    }
  }

  get nextPlayer(): Player | undefined {
    if (this.matchID === undefined) {

    } else if (this.subkey1 === undefined) {
      return Player.One

    } else if (this.subkey2 === undefined) {
      return Player.Two

    } else if (this.reply !== undefined) {
      return this.state.nextPlayer()

    } else if (this.commit !== undefined) {
      switch (this.state.nextPlayer()) {
      case Player.One:
        return Player.Two

      case Player.Two:
        return Player.One
      }

    } else {
      return this.state.nextPlayer()
    }
  }

  next(message: Message): State {
    const next = this.copy()

    if (next.hash !== undefined && ethers.utils.hexlify(message.encoding.subarray(65, 65 + 32)) !== next.hash) {
      throw Error(`next.hash !== undefined && ethers.utils.hexlify(message.encoding.subarray(65, 65 + 32)) !== next.hash`)
    }

    next.hash = message.hash

    if (next.matchID === undefined) {
      if (message.author !== Matcher) {
        throw Error(`message.author !== Matcher`)
      }

      if (message.message.length !== 16 + 2 * 20) {
        throw Error(`message.message.length !== 16 + 2 * 20`)
      }

      next.matchID = message.message.subarray(0, 16)
      next.account1 = ethers.utils.getAddress(ethers.utils.hexlify(message.message.subarray(16, 16 + 20)))
      next.account2 = ethers.utils.getAddress(ethers.utils.hexlify(message.message.subarray(16 + 20)))

    } else if (next.subkey1 === undefined) {
      if (message.author !== next.account1) {
        throw Error(`message.author !== next.account1`)
      }

      if (message.message.length !== 20) {
        throw Error(`message.message.length !== 20`)
      }

      next.subkey1 = ethers.utils.getAddress(ethers.utils.hexlify(message.message))

    } else if (next.subkey2 === undefined) {
      if (message.author !== next.account2) {
        throw Error(`message.author !== next.account2`)
      }

      if (message.message.length !== 20) {
        throw Error(`message.message.length !== 20`)
      }

      next.subkey2 = ethers.utils.getAddress(ethers.utils.hexlify(message.message))

      next.state = new bindings.State()

    } else if (next.reply !== undefined) {
      switch (next.state.nextPlayer()) {
      case Player.One:
        if (message.author !== next.subkey1) {
          throw Error(`message.author !== next.subkey1`)
        }

        break

      case Player.Two:
        if (message.author !== next.subkey2) {
          throw Error(`message.author !== next.subkey2`)
        }

        break

      default:
        throw Error(`next.state.nextPlayer() === undefined`)
      }

      if (message.message.length !== 16) {
        throw Error(`message.message.length !== 16`)
      }

      if (ethers.utils.keccak256(message.message) !== ethers.utils.hexlify(next.commit.subarray(0, 32))) {
        throw Error(`ethers.utils.keccak256(message.message) !== ethers.utils.hexlify(next.commit.subarray(0, 32))`)
      }

      const random = next.reply.slice()

      for (let i = 0; i < random.length; i++) {
        random[i] ^= message.message[i]
      }

      next.state = next.state.next(next.state.nextPlayer(), next.commit.subarray(32), random)
      delete next.commit
      delete next.reply

    } else if (next.commit !== undefined) {
      switch (next.state.nextPlayer()) {
      case Player.One:
        if (message.author !== next.subkey2) {
          throw Error(`message.author !== next.subkey2`)
        }

        break

      case Player.Two:
        if (message.author !== next.subkey1) {
          throw Error(`message.author !== next.subkey1`)
        }

        break

      default:
        throw Error(`next.state.nextPlayer() === undefined`)
      }

      if (message.message.length !== 16) {
        throw Error(`message.message.length !== 16`)
      }

      next.reply = message.message

    } else {
      switch (next.state.nextPlayer()) {
      case Player.One:
        if (message.author !== next.subkey1) {
          throw Error(`message.author !== next.subkey1`)
        }

        break

      case Player.Two:
        if (message.author !== next.subkey2) {
          throw Error(`message.author !== next.subkey2`)
        }

        break

      default:
        throw Error(`next.state.nextPlayer() === undefined`)
      }

      if (message.message.length < 32) {
        throw Error(`message.message.length < 32`)
      }

      next.commit = message.message
    }

    return next
  }

  private hash?: string
  private matchID?: Uint8Array
  private account1?: string
  private account2?: string
  private subkey1?: string
  private subkey2?: string
  private commit?: Uint8Array
  private reply?: Uint8Array
  private state?: bindings.State

  private copy(): State {
    const copy = new State()

    copy.hash = this.hash
    copy.matchID = this.matchID
    copy.account1 = this.account1
    copy.account2 = this.account2
    copy.subkey1 = this.subkey1
    copy.subkey2 = this.subkey2
    copy.commit = this.commit
    copy.reply = this.reply
    copy.state = this.state

    return copy
  }
}

import Player = bindings.Player

export interface Message {
  readonly message: Uint8Array
  readonly author: string
  readonly encoding: Uint8Array
  readonly hash: string
}

export function createRootMessage(matchID: ethers.utils.Arrayish, account1: ethers.utils.Arrayish, account2: ethers.utils.Arrayish, signer: ethers.Signer): Promise<Message> {
  const matchIDBytes = ethers.utils.arrayify(matchID)

  if (matchIDBytes.length != 16) {
    throw Error(`matchIDBytes.length != 16`)
  }

  const account1Bytes = ethers.utils.arrayify(account1)

  if (account1Bytes.length != 20) {
    throw Error(`account1Bytes.length != 20`)
  }

  const account2Bytes = ethers.utils.arrayify(account2)

  if (account2Bytes.length != 20) {
    throw Error(`account2Bytes.length != 20`)
  }

  const message = new Uint8Array(matchIDBytes.length + account1Bytes.length + account2Bytes.length)
  message.set(matchIDBytes)
  message.set(account1Bytes, matchIDBytes.length)
  message.set(account2Bytes, matchIDBytes.length + account1Bytes.length)

  return createMessage(message, signer)
}

export function createSubkeyMessage(subkey: ethers.utils.Arrayish, signer: ethers.Signer, parent: Message): Promise<Message> {
  const subkeyBytes = ethers.utils.arrayify(subkey)

  if (subkeyBytes.length != 20) {
    throw Error(`subkeyBytes.length != 20`)
  }

  return createMessage(subkeyBytes, signer, parent)
}

export async function createActionMessage(action: ethers.utils.Arrayish, signer: ethers.Signer, parent: Message): Promise<[Message, (reply: Message) => Promise<Message>]> {
  const actionBytes = ethers.utils.arrayify(action)

  const random = ethers.utils.randomBytes(16)
  const hash = ethers.utils.arrayify(ethers.utils.keccak256(random))

  const message = new Uint8Array(hash.length + actionBytes.length)
  message.set(hash)
  message.set(actionBytes, hash.length)

  const commit = await createMessage(message, signer, parent)

  return [commit, (reply: Message) => {
    if (reply.message.length != 16) {
      throw Error(`reply.message.length != 16`)
    }

    const parent = ethers.utils.hexlify(reply.encoding.subarray(65, 65 + 32))

    if (parent !== commit.hash) {
      throw Error(`parent !== commit.hash`)
    }

    return createMessage(random, signer, reply)
  }]
}

export function createReplyMessage(signer: ethers.Signer, parent: Message): Promise<Message> {
  return createMessage(ethers.utils.randomBytes(16), signer, parent)
}

export function decodeMessage(encoding: ethers.utils.Arrayish, parent?: Message): [Message, Uint8Array] {
  const encodingBytes = ethers.utils.arrayify(encoding)
  const length = new DataView(encodingBytes.buffer).getUint32(65 + 32, true)

  if (encodingBytes.length < 65 + 32 + 4 + length) {
    throw Error(`encodingBytes.length < 65 + 32 + 4 + length`)
  }

  const hash = ethers.utils.hexlify(encodingBytes.subarray(65, 65 + 32))

  if (parent === undefined) {
    if (hash !== ethers.constants.HashZero) {
      throw Error('hash !== ethers.constants.HashZero')
    }

  } else {
    if (hash !== parent.hash) {
      throw Error(`hash !== parent.hash`)
    }
  }

  const head = encodingBytes.subarray(0, 65 + 32 + 4 + length)
  const tail = encodingBytes.subarray(65 + 32 + 4 + length)
  const message = head.subarray(65 + 32 + 4)
  const signature = ethers.utils.hexlify(head.subarray(0, 65))

  return [
    {
      message,
      author: ethers.utils.verifyMessage(head.subarray(65), signature),
      encoding: head,
      hash: ethers.utils.keccak256(head)
    },
    tail
  ]
}

async function createMessage(message: ethers.utils.Arrayish, signer: ethers.Signer, parent?: Message): Promise<Message> {
  const messageBytes = ethers.utils.arrayify(message)

  const encoding = new Uint8Array(65 + 32 + 4 + messageBytes.length)
  new DataView(encoding.buffer).setUint32(65 + 32, messageBytes.length, true)
  encoding.set(messageBytes, 65 + 32 + 4)

  if (parent !== undefined) {
    encoding.set(ethers.utils.arrayify(parent.hash), 65)
  }

  const signature = await signer.signMessage(encoding.subarray(65))
  encoding.set(ethers.utils.arrayify(signature))

  return {
    message: messageBytes,
    author: ethers.utils.verifyMessage(encoding.subarray(65), signature),
    encoding,
    hash: ethers.utils.keccak256(encoding)
  }
}
