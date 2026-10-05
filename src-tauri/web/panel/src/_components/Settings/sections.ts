export type GlyphName =
  | "music" | "display" | "vote" | "tray" | "book" | "gear" | "pulse" | "info"
  | "weather" | "clock" | "megaphone" | "shield" | "globe" | "key" | "lock" | "trash"
  | "code" | "plug" | "lastfm" | "spotify" | "external" | "chevron" | "back" | "check"
  | "search" | "close" | "copy" | "refresh" | "puzzle" | "slides"

export type Tint = "lastfm" | "spotify" | "indigo" | "slate"

export const Tints: Record<Tint, [string, string]> = {
  lastfm: ["#f0413a", "#c3110b"],
  spotify: ["#35d46e", "#179443"],
  indigo: ["#8d8aff", "#5553d8"],
  slate: ["#8e97a8", "#5b6475"],
}

export type SectionId = "scrobbling" | "screen" | "slides" | "voting" | "cloudflare" | "plugins" | "management" | "performance" | "about"

export interface Anchor {
  Id: string
  Label: string
  Keys: string[]
}

export interface Section {
  Id: SectionId
  Label: string
  Description: string
  Icon: GlyphName
  Keys: string[]
  Anchors: Anchor[]
}

export const Sections: Section[] = [
  {
    Id: "scrobbling",
    Label: "settings.music",
    Description: "settings.music.description",
    Icon: "music",
    Keys: ["settings.music", "settings.scrobbling", "settings.scrobbling.source", "settings.spotify.lastfm", "settings.spotify.spotify", "settings.spotify.api", "settings.serial.method"],
    Anchors: [
      { Id: "scrobbling-lessons", Label: "settings.scrobbling.lessons.short", Keys: ["settings.scrobbling.lessons", "settings.scrobbling.lessons.description"] },
    ],
  },
  {
    Id: "screen",
    Label: "settings.screen",
    Description: "settings.screen.description",
    Icon: "display",
    Keys: ["settings.screen"],
    Anchors: [
      { Id: "screen-pages", Label: "settings.pages", Keys: ["settings.pages", "settings.pages.lesson", "settings.pages.news", "settings.pages.cycle"] },
      { Id: "screen-weather", Label: "settings.weather", Keys: ["settings.weather", "settings.weather.location"] },
      { Id: "screen-lessons", Label: "settings.lessons", Keys: ["settings.lessons", "settings.lessons.duration", "settings.lessons.times", "settings.lessons.remote"] },
      { Id: "screen-notice", Label: "settings.notice", Keys: ["settings.notice", "settings.notice.footer"] },
    ],
  },
  {
    Id: "slides",
    Label: "settings.slides",
    Description: "settings.slides.description",
    Icon: "slides",
    Keys: ["settings.slides", "settings.slides.collections", "settings.slides.new"],
    Anchors: [],
  },
  {
    Id: "voting",
    Label: "voting",
    Description: "voting.description",
    Icon: "vote",
    Keys: ["voting", "voting.enabled", "voting.review"],
    Anchors: [
      { Id: "voting-page", Label: "voting.page", Keys: ["voting.page", "voting.url"] },
      { Id: "voting-captcha", Label: "voting.captcha", Keys: ["voting.captcha", "voting.captcha.description", "voting.captcha.sitekey", "voting.captcha.secret"] },
    ],
  },
  {
    Id: "cloudflare",
    Label: "settings.cloudflare",
    Description: "settings.cloudflare.description",
    Icon: "globe",
    Keys: ["settings.cloudflare", "settings.cloudflare.management", "settings.cloudflare.voting", "settings.cloudflare.addresses"],
    Anchors: [],
  },
  {
    Id: "plugins",
    Label: "settings.plugins",
    Description: "settings.plugins.description",
    Icon: "puzzle",
    Keys: ["settings.plugins", "settings.plugins.installed", "settings.plugins.rescan", "settings.plugins.safe.link"],
    Anchors: [],
  },
  {
    Id: "management",
    Label: "settings.management",
    Description: "settings.management.description",
    Icon: "gear",
    Keys: ["settings.management", "settings.password", "settings.password.change", "settings.nuke"],
    Anchors: [],
  },
  {
    Id: "performance",
    Label: "settings.performance",
    Description: "settings.performance.description",
    Icon: "pulse",
    Keys: ["settings.performance", "settings.performance.cpu", "settings.performance.memory", "settings.performance.network", "settings.performance.server", "settings.performance.searches", "settings.performance.fps"],
    Anchors: [],
  },
  {
    Id: "about",
    Label: "settings.about",
    Description: "settings.about.description",
    Icon: "info",
    Keys: ["settings.about", "settings.about.version", "settings.about.address", "settings.about.uptime", "settings.about.database"],
    Anchors: [],
  },
]

export function SectionOf(Id: string): Section | null {
  return Sections.find(S => S.Id === Id) ?? null
}

export function OwnerOf(Id: string): Section | null {
  return Sections.find(S => S.Id === Id || S.Anchors.some(A => A.Id === Id)) ?? null
}
