import * as ethers from 'ethers'

export type Bytes = ethers.utils.Arrayish | ethers.utils.Hexable

export class Message {
  constructor(message: Message | Bytes) {
    if (message instanceof Message) {
      this.author = message.author
      this.data = message.data
      this.encoding = message.encoding
      this.hash = message.hash
      this.parent = message.parent
    } else {
      this.encoding = ethers.utils.arrayify(message)
      if (this.encoding.length < 65 + 32 + 4) {
        throw Error(`this.encoding.length < 65 + 32 + 4`)
      }

      const dataLength = new DataView(
        this.encoding.buffer,
        this.encoding.byteOffset,
        this.encoding.length
      ).getUint32(65 + 32, true)

      if (this.encoding.length !== 65 + 32 + 4 + dataLength) {
        throw Error(`this.encoding.length !== 65 + 32 + 4 + dataLength`)
      }

      const signature = ethers.utils.hexlify(this.encoding.subarray(0, 65))
      this.author = ethers.utils.verifyMessage(
        this.encoding.subarray(65),
        signature
      )

      this.data = this.encoding.subarray(65 + 32 + 4)
      this.hash = ethers.utils.keccak256(this.encoding)
      this.parent = ethers.utils.hexlify(this.encoding.subarray(65, 65 + 32))
    }
  }

  readonly author: string
  readonly data: Uint8Array
  readonly encoding: Uint8Array
  readonly hash: string
  readonly parent: string
}

export async function createMessage(
  data: Bytes,
  author: ethers.Signer,
  parent?: Message | Bytes
): Promise<Message> {
  const dataBytes = ethers.utils.arrayify(data)

  const encoding = new Uint8Array(65 + 32 + 4 + dataBytes.length)

  new DataView(encoding.buffer, encoding.byteOffset, encoding.length).setUint32(
    65 + 32,
    dataBytes.length,
    true
  )

  encoding.set(dataBytes, 65 + 32 + 4)

  if (parent !== undefined) {
    encoding.set(ethers.utils.arrayify(new Message(parent).hash), 65)
  }

  encoding.set(
    ethers.utils.arrayify(await author.signMessage(encoding.subarray(65)))
  )

  return new Message(encoding)
}
