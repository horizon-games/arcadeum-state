import * as bindings from 'bindings'

;(() => {
  try {
    let state = bindings.State.new()
    log(state)
    state = state.next(1, new Uint8Array([0, 0]), new Uint8Array(16))
    log(state)
    state = state.next(2, new Uint8Array([1, 0]), new Uint8Array(16))
    log(state)
    state = state.next(1, new Uint8Array([0, 1]), new Uint8Array(16))
    log(state)
    state = state.next(2, new Uint8Array([1, 1]), new Uint8Array(16))
    log(state)
    state = state.next(1, new Uint8Array([0, 2]), new Uint8Array(16))
    log(state)
    state = state.next(2, new Uint8Array([1, 2]), new Uint8Array(16))
    log(state)
  } catch (e) {
    console.log(`error: ${e}`)
  }
})()

function log(state: bindings.State): void {
  console.log(state)
  console.log(`winner: ${state.winner()}`)
  console.log(`next player: ${state.next_player()}`)
  console.log()
}
