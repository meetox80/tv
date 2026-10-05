import { Sections, type GlyphName } from "./sections"

export interface Hit {
  Id: string
  Kind: "section" | "anchor" | "review" | "docs"
  Label: string
  Context: string
  Icon: GlyphName
}

const _Actions: { Id: string; Kind: "review" | "docs"; Label: string; Icon: GlyphName; Keys: string[] }[] = [
  { Id: "review", Kind: "review", Label: "voting.review", Icon: "tray", Keys: ["voting.review", "voting.review.row", "voting.counts.pending"] },
  { Id: "docs", Kind: "docs", Label: "voting.docs", Icon: "book", Keys: ["voting.docs"] },
]

export function Fold(Text: string): string {
  return Text
    .toLowerCase()
    .replace(/ł/g, "l")
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
}

export function Search(Raw: string, T: (Key: string) => string): Hit[] {
  const Query = Fold(Raw.trim())
  if (!Query) return []

  const Score = (Keys: string[]): number => {
    let Best = 0
    for (const K of Keys) {
      const Text = Fold(T(K))
      if (Text.startsWith(Query)) Best = Math.max(Best, 3)
      else if (Text.split(/\s+/).some(W => W.startsWith(Query))) Best = Math.max(Best, 2)
      else if (Text.includes(Query)) Best = Math.max(Best, 1)
    }
    return Best
  }

  const Ranked: { Hit: Hit; Score: number; Order: number }[] = []
  const Put = (Hit: Hit, Value: number): void => {
    if (Value > 0) Ranked.push({ Hit, Score: Value, Order: Ranked.length })
  }

  for (const S of Sections) {
    Put({ Id: S.Id, Kind: "section", Label: T(S.Label), Context: "", Icon: S.Icon }, Score([S.Label, ...S.Keys]))
    for (const A of S.Anchors) {
      Put({ Id: A.Id, Kind: "anchor", Label: T(A.Label), Context: T(S.Label), Icon: S.Icon }, Score([A.Label, ...A.Keys]))
    }
  }

  for (const A of _Actions) {
    Put({ Id: A.Id, Kind: A.Kind, Label: T(A.Label), Context: "", Icon: A.Icon }, Score([A.Label, ...A.Keys]))
  }

  return Ranked.sort((A, B) => B.Score - A.Score || A.Order - B.Order).map(R => R.Hit)
}
