/*
 * Arcadeum blockchain game framework
 * Copyright (C) 2019  Horizon Blockchain Games Inc.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 3.0 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

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
