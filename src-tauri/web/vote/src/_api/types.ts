export type SongStatus = "pending" | "approved" | "added" | "rejected"

export type SearchSource = "spotify" | "lastfm" | "deezer"

export type Ballot = -1 | 0 | 1

export interface Device {
  Handle: string
  Banned: boolean
  Left: number
  Since: number
}

export interface AppState {
  Enabled: boolean
  SiteKey: string | null
  Limit: number
  ResetIn: number
  Device: Device | null
}

export interface Track {
  Provider: SearchSource
  Id: string
  Title: string
  Artist: string
  Album: string | null
  Cover: string | null
  Explicit: boolean | null
  DurationMs: number | null
  Status: SongStatus | null
}

export interface SearchResult {
  Source: SearchSource
  Results: Track[]
}

export interface BoardSong {
  Id: number
  Title: string
  Artist: string
  Album: string | null
  Cover: string | null
  Explicit: boolean | null
  Score: number
  Vote: Ballot
}

export interface MySong {
  Id: number
  Title: string
  Artist: string
  Album: string | null
  Cover: string | null
  Explicit: boolean | null
  Status: SongStatus
  Created: number
}

export interface Songs {
  Board: BoardSong[]
  Mine: MySong[]
}

export interface RequestCreated {
  Song: MySong
  Left: number
}

export interface VoteResult {
  Score: number
  Vote: Ballot
}
