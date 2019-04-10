export interface Game {
  owner(): Uint8Array
  new (
    matchSeed: Uint8Array,
    publicSeed1: Uint8Array,
    publicSeed2: Uint8Array,
    player?: number,
    secretSeed?: Uint8Array,
    logger?: (message: any) => void,
    listener?: () => void,
    sender?: (message: number[]) => void,
    seeder?: (length: number) => number[]
  ): Match
}

export interface Match {
  free(): void
  player(): number | undefined
  sharedState(): any
  localState(): any
  winner(): number | undefined
  nextPlayer(): number | undefined
  mutate(player: number, action: Uint8Array): void
}

export enum Player {
  One = 0,
  Two = 1
}
