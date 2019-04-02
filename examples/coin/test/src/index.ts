import * as arcadeum from 'arcadeum-bindings'
import * as coin from 'arcadeum-coin'
import * as child_process from 'child_process'
import * as ethers from 'ethers'
import * as path from 'path'

const owner = ethers.Wallet.fromMnemonic(
  `winter off snap small sleep debate cheap drill elevator glove caution once`
)

const client1 = child_process.fork(path.join(__dirname, `client-async`))
const client2 = child_process.fork(path.join(__dirname, `client-sync`))

const send = (player: arcadeum.Player, message: arcadeum.Message) => {
  switch (player) {
    case arcadeum.Player.One:
      console.log(
        `server (${process.pid}) > client 1 (${
          client1.pid
        }): ${ethers.utils.hexlify(message.encoding)}`
      )

      client1.send(ethers.utils.hexlify(message.encoding))
      break

    case arcadeum.Player.Two:
      console.log(
        `server (${process.pid}) > client 2 (${
          client2.pid
        }): ${ethers.utils.hexlify(message.encoding)}`
      )

      client2.send(ethers.utils.hexlify(message.encoding))
      break
  }
}

const server = {
  account1: undefined as string | undefined,
  account2: undefined as string | undefined,
  server: undefined as arcadeum.Server | undefined
}

client1.on(`message`, (message: any) => {
  console.log(`server (${process.pid}) < client 1 (${client1.pid}): ${message}`)

  if (server.account1 === undefined) {
    server.account1 = message

    if (server.account2 !== undefined) {
      server.server = new arcadeum.Server(
        coin.Store,
        owner,
        server.account1,
        server.account2,
        send
      )
    }
  } else {
    server.server.receive(ethers.utils.arrayify(message))
  }
})

client2.on(`message`, (message: any) => {
  console.log(`server (${process.pid}) < client 2 (${client2.pid}): ${message}`)

  if (server.account2 === undefined) {
    server.account2 = message

    if (server.account1 !== undefined) {
      server.server = new arcadeum.Server(
        coin.Store,
        owner,
        server.account1,
        server.account2,
        send
      )
    }
  } else {
    server.server.receive(ethers.utils.arrayify(message))
  }
})
