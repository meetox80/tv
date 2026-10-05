import { Get, Post } from "./http"
import type {
  AppState, Ballot, Device, RequestCreated, SearchResult, SearchSource, Songs, VoteResult,
} from "./types"

export function LoadState(): Promise<AppState> {
  return Get<AppState>("/api/state")
}

export function RegisterDevice(Captcha: string): Promise<Device> {
  return Post<Device>("/api/device", { Captcha })
}

export function Search(Query: string, Signal: AbortSignal): Promise<SearchResult> {
  return Get<SearchResult>(`/api/search?q=${encodeURIComponent(Query)}`, Signal)
}

export function SubmitRequest(Provider: SearchSource, Id: string): Promise<RequestCreated> {
  return Post<RequestCreated>("/api/requests", { Provider, Id })
}

export function LoadSongs(): Promise<Songs> {
  return Get<Songs>("/api/songs")
}

export function CastVote(SongId: number, Value: Ballot): Promise<VoteResult> {
  return Post<VoteResult>(`/api/songs/${SongId}/vote`, { Value })
}
