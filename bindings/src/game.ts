export interface Game {
  owner(): Uint8Array
  new (arg0: number | undefined, arg1: any, arg2: any, arg3: any): Match
}

export interface Match {
  free(): void
  player(): number | undefined
  sharedState(): any
  localState(): any
  winner(): number | undefined
  nextPlayer(): number | undefined
  mutate(arg0: number, arg1: Uint8Array): void
}

export enum Player {
  One = 0,
  Two = 1
}
