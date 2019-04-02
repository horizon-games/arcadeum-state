import * as ethers from 'ethers'

import { Game, Match, Player } from './game'
import { Bytes, Message, createMessage } from './message'

export class Server {
  constructor(
    private readonly game: Game,
    owner: ethers.Signer,
    private readonly account1: string,
    private readonly account2: string,
    private readonly send: (player: Player, message: Message) => void
  ) {
    const account1Bytes = ethers.utils.arrayify(this.account1)
    if (account1Bytes.length !== 20) {
      throw Error(`account1Bytes.length !== 20`)
    }
    this.account1 = ethers.utils.getAddress(this.account1)

    const account2Bytes = ethers.utils.arrayify(this.account2)
    if (account2Bytes.length !== 20) {
      throw Error(`account2Bytes.length !== 20`)
    }
    this.account2 = ethers.utils.getAddress(this.account2)

    this.match = new this.game()

    this.messages = []

    const rootMessage = new Uint8Array(16 + 2 * 20)
    rootMessage.set(account1Bytes, 16)
    rootMessage.set(account2Bytes, 16 + 20)

    createMessage(rootMessage, owner).then((rootMessage: Message) => {
      this.messages.push(rootMessage)

      this.send(Player.One, rootMessage)
      this.send(Player.Two, rootMessage)
    })
  }

  receive(messageBytes: Message | Bytes) {
    const message = new Message(messageBytes)

    if (message.parent !== this.lastMessage.hash) {
      throw Error(`message.parent !== this.lastMessage.hash`)
    }

    if (this.subkey1 === undefined) {
      if (message.author !== this.account1) {
        throw Error(`message.author !== this.account1`)
      }

      if (message.data.length !== 20) {
        throw Error(`message.data.length !== 20`)
      }

      this.subkey1 = ethers.utils.getAddress(ethers.utils.hexlify(message.data))
      this.messages.push(message)
      this.send(Player.Two, message)
    } else if (this.subkey2 === undefined) {
      if (message.author !== this.account2) {
        throw Error(`message.author !== this.account2`)
      }

      if (message.data.length !== 20) {
        throw Error(`message.data.length !== 20`)
      }

      this.subkey2 = ethers.utils.getAddress(ethers.utils.hexlify(message.data))
      this.messages.push(message)
      this.send(Player.One, message)
    } else {
      const length = this.messages.length

      try {
        switch (message.author) {
          case this.subkey1:
            this.messages.push(message)
            this.match.mutate(Player.One, message.data)
            this.send(Player.Two, message)
            break

          case this.subkey2:
            this.messages.push(message)
            this.match.mutate(Player.Two, message.data)
            this.send(Player.One, message)
            break

          default:
            throw Error(
              `message.author !== this.subkey1 && message.author !== this.subkey2`
            )
        }
      } catch (error) {
        this.messages.length = length
        throw error
      }
    }
  }

  private readonly match: Match

  private subkey1?: string
  private subkey2?: string

  private readonly messages: Message[]
  private get lastMessage(): Message {
    return this.messages[this.messages.length - 1]
  }
}
