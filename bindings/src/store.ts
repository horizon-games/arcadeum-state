import * as ethers from 'ethers'

import { Game, Match, Player } from './game'
import { Bytes, Message, createMessage } from './message'

export class Store {
  constructor(
    game: Game,
    rootMessageBytes: Message | Bytes,
    private readonly account: ethers.Signer,
    log: (message: any) => void,
    private readonly send: (message: Message) => void
  ) {
    const rootMessage = new Message(rootMessageBytes)

    const owner = ethers.utils.getAddress(ethers.utils.hexlify(game.owner()))
    if (rootMessage.author !== owner) {
      throw Error(`rootMessage.author !== owner`)
    }

    if (
      rootMessage.parent !==
      `0x0000000000000000000000000000000000000000000000000000000000000000`
    ) {
      throw Error(
        'rootMessage.parent !== `0x0000000000000000000000000000000000000000000000000000000000000000`'
      )
    }

    if (rootMessage.data.length !== 16 + 2 * 20) {
      throw Error(`rootMessage.data.length !== 16 + 2 * 20`)
    }

    this.account1 = ethers.utils.getAddress(
      ethers.utils.hexlify(rootMessage.data.subarray(16, 16 + 20))
    )

    this.account2 = ethers.utils.getAddress(
      ethers.utils.hexlify(rootMessage.data.subarray(16 + 20))
    )

    this.subkey = ethers.Wallet.createRandom()

    this.messages = [Promise.resolve(rootMessage)]

    this.ready = new Promise(
      (resolve: () => void, reject: (reason: any) => void) => {
        this.resolveReady = () => {
          delete this.resolveReady
          delete this.rejectReady

          resolve()

          for (const index in this.listeners) {
            this.listeners[index]()
          }
        }

        this.rejectReady = (reason: any) => {
          delete this.resolveReady
          delete this.rejectReady

          reject(reason)
        }

        this.account
          .getAddress()
          .then(async (address: string) => {
            const listener = () => {
              for (const index in this.listeners) {
                setImmediate(this.listeners[index])
              }
            }

            const sender = (data: number[]) => {
              this.messages.push(
                this.lastMessage.then(async (lastMessage: Message) => {
                  const message = await createMessage(
                    data,
                    this.subkey,
                    lastMessage
                  )

                  this.send(message)
                  return message
                })
              )
            }

            const seeder = (length: number) =>
              Array.from(ethers.utils.randomBytes(length))

            switch (address) {
              case this.account1:
                this.match = new game(Player.One, log, listener, sender, seeder)

                this.subkey1 = this.subkey.address

                this.messages.push(
                  this.lastMessage.then(async (lastMessage: Message) => {
                    const subkeyMessage = await createMessage(
                      this.subkey1,
                      this.account,
                      lastMessage
                    )

                    this.send(subkeyMessage)
                    return subkeyMessage
                  })
                )

                break

              case this.account2:
                this.match = new game(Player.Two, log, listener, sender, seeder)
                break

              default:
                throw Error(
                  `address !== this.account1 && address !== this.account2`
                )
            }
          })
          .catch((error: any) => {
            this.rejectReady(error)
          })
      }
    )

    this.listeners = {}
    this.nextListener = 0
  }

  readonly ready: Promise<void>

  get player(): Player | undefined {
    if (this.match !== undefined) {
      return this.match.player()
    }
  }

  get state(): { shared: any; local: any } | undefined {
    if (this.match !== undefined) {
      return {
        shared: this.sharedState,
        local: this.localState
      }
    }
  }

  get sharedState(): any {
    if (this.match !== undefined) {
      return this.match.sharedState()
    }
  }

  get localState(): any {
    if (this.match !== undefined) {
      return this.match.localState()
    }
  }

  get winner(): Player | undefined {
    if (this.match !== undefined) {
      return this.match.winner()
    }
  }

  get nextPlayer(): Player | undefined {
    if (this.match !== undefined) {
      return this.match.nextPlayer()
    }
  }

  getState(): { shared: any; local: any } | undefined {
    return this.state
  }

  subscribe(listener: () => void): () => void {
    const index = this.nextListener

    this.listeners[index] = listener
    this.nextListener++

    return () => delete this.listeners[index]
  }

  get opponentActions(): Promise<void> {
    const resolve = () => {
      if (this.nextPlayer === this.player || this.nextPlayer === undefined) {
        return Promise.resolve()
      }

      return new Promise(
        (resolve: (value: void) => void, reject: (reason: any) => void) => {
          const unsubscribe = { unsubscribe: undefined }

          unsubscribe.unsubscribe = this.subscribe(() => {
            if (
              this.nextPlayer === this.player ||
              this.nextPlayer === undefined
            ) {
              unsubscribe.unsubscribe()

              resolve()
            }
          })
        }
      )
    }

    if (this.resolveReady !== undefined) {
      return this.ready.then(resolve)
    } else {
      return resolve()
    }
  }

  dispatch(action: Bytes) {
    return new Promise((resolve: () => void, reject: (reason: any) => void) => {
      const unsubscribe = { unsubscribe: undefined }

      unsubscribe.unsubscribe = this.subscribe(() => {
        unsubscribe.unsubscribe()

        resolve()
      })

      this.match.mutate(this.player, ethers.utils.arrayify(action))
    })
  }

  async receive(messageBytes: Message | Bytes) {
    const message = new Message(messageBytes)

    if (message.parent !== (await this.lastMessage).hash) {
      throw Error(`message.parent !== (await this.lastMessage).hash`)
    }

    if (this.subkey1 === undefined) {
      if (message.author !== this.account1) {
        throw Error(`message.author !== this.account1`)
      }

      if (message.data.length !== 20) {
        throw Error(`message.data.length !== 20`)
      }

      this.subkey1 = ethers.utils.getAddress(ethers.utils.hexlify(message.data))
      this.messages.push(Promise.resolve(message))

      this.subkey2 = this.subkey.address
      this.messages.push(
        this.lastMessage.then(async (lastMessage: Message) => {
          const subkeyMessage = await createMessage(
            this.subkey2,
            this.account,
            lastMessage
          )

          this.send(subkeyMessage)
          return subkeyMessage
        })
      )

      this.resolveReady()
    } else if (this.subkey2 === undefined) {
      if (message.author !== this.account2) {
        throw Error(`message.author !== this.account2`)
      }

      if (message.data.length !== 20) {
        throw Error(`message.data.length !== 20`)
      }

      this.subkey2 = ethers.utils.getAddress(ethers.utils.hexlify(message.data))
      this.messages.push(Promise.resolve(message))

      this.resolveReady()
    } else {
      const length = this.messages.length

      switch (message.author) {
        case this.subkey1:
          this.messages.push(Promise.resolve(message))

          try {
            this.match.mutate(Player.One, message.data)
          } catch (error) {
            this.messages.length = length
          }

          break

        case this.subkey2:
          this.messages.push(Promise.resolve(message))

          try {
            this.match.mutate(Player.Two, message.data)
          } catch (error) {
            this.messages.length = length
          }

          break

        default:
          throw Error(
            `message.author !== this.subkey1 && message.author !== this.subkey2`
          )
      }
    }
  }

  get proof(): Promise<Uint8Array> {
    return Promise.all(this.messages).then((messages: Message[]) => {
      let length = 0
      for (const message of messages) {
        length += message.encoding.length
      }

      const proof = new Uint8Array(length)

      let offset = 0
      for (const message of messages) {
        proof.set(message.encoding, offset)
        offset += message.encoding.length
      }

      return proof
    })
  }

  private readonly account1: string
  private readonly account2: string

  private readonly subkey: ethers.Wallet

  private subkey1?: string
  private subkey2?: string

  private match?: Match

  private readonly messages: Promise<Message>[]
  private get lastMessage(): Promise<Message> {
    return this.messages[this.messages.length - 1]
  }

  private resolveReady?: () => void
  private rejectReady?: (reason: any) => void

  private readonly listeners: { [index: number]: () => void }
  private nextListener: number
}
