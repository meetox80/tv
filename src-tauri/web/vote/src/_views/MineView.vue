<script setup lang="ts">
import { computed } from "vue"
import Blank from "../_components/Blank.vue"
import Cover from "../_components/Cover.vue"
import Explicit from "../_components/Explicit.vue"
import { When } from "../_api/format"
import type { MySong, SongStatus } from "../_api/types"
import { ActiveTab, Board, Device, GoVote, Mine, SongsReady } from "../_state/store"

interface Step {
  Label: string
  State: "done" | "waiting" | "failed" | "ahead"
}

const _Reached: Record<SongStatus, number> = { pending: 1, approved: 2, added: 3, rejected: 1 }

const _Short: Record<SongStatus, string> = {
  pending: "Oczekuje",
  approved: "W rankingu",
  added: "Na playliście",
  rejected: "Odrzucona",
}

const Vacant = computed(() => SongsReady.value && !Mine.value.length)
const Latest = computed(() => Mine.value[0] ?? null)
const Older = computed(() => Mine.value.slice(1))

function Steps(Status: SongStatus): Step[] {
  const Reached = _Reached[Status]
  const Middle = Status === "rejected" ? "Odrzucona" : Status === "pending" ? "Czeka na decyzję" : "W rankingu"
  const Labels = ["Wysłana", Middle, "Na playliście"]

  return Labels.map((Label, Index) => {
    if (Status === "rejected" && Index === 1) return { Label, State: "failed" }
    if (Status === "pending" && Index === 1) return { Label, State: "waiting" }
    return { Label, State: Index < Reached ? "done" : "ahead" }
  })
}

function Segments(Status: SongStatus): string[] {
  return [0, 1, 2].map((Index) => {
    if (Status === "rejected" && Index === 1) return "failed"
    return Index < _Reached[Status] ? "done" : "ahead"
  })
}

function Open(Song: MySong): void {
  if (Song.Status !== "approved") return
  const Hit = Board.value.find(B => B.Id === Song.Id)
  GoVote(Hit ? Hit.Id : null)
}
</script>

<template>
  <section class="View" aria-labelledby="mine-heading">
    <h1 id="mine-heading" :class="Vacant ? 'SrOnly' : 'Heading'">Twoje propozycje</h1>

    <div v-if="!SongsReady" class="Layout" aria-hidden="true">
      <div class="Latest Glass">
        <span class="Art Skeleton" />
        <span class="Info">
          <span class="Skeleton GhostLine" style="width: 70%" />
          <span class="Skeleton GhostLine Short" />
        </span>
      </div>
    </div>

    <Blank
      v-else-if="Vacant"
      Title="Nie masz jeszcze propozycji"
      Text="Wyślij utwór, a tutaj zobaczysz, co się z nim dzieje."
    >
      <button type="button" @click="ActiveTab = 'request'">Zaproponuj utwór</button>
    </Blank>

    <div v-else-if="Latest" class="Layout">
      <article class="Latest Glass">
        <div class="Top">
          <div class="Art">
            <Cover :Src="Latest.Cover" :Title="Latest.Title" :Size="140" Fill />
          </div>

          <div class="Info">
            <p class="LatestName">
              <span>{{ Latest.Title }}</span>
              <Explicit v-if="Latest.Explicit" />
            </p>
            <p class="LatestBy">{{ Latest.Artist }}</p>
            <p class="LatestWhen">{{ When(Latest.Created) }}</p>
          </div>
        </div>

        <ol class="Track" :aria-label="`Status: ${_Short[Latest.Status]}`">
          <li v-for="(S, Index) in Steps(Latest.Status)" :key="Index" class="Step" :class="S.State">
            <span class="Node" aria-hidden="true">
              <svg v-if="S.State === 'done'" viewBox="0 0 24 24"><path d="m6 12.5 4 4 8-9" /></svg>
              <svg v-else-if="S.State === 'failed'" viewBox="0 0 24 24"><path d="M8 8l8 8M16 8l-8 8" /></svg>
            </span>
            <span class="StepLabel">{{ S.Label }}</span>
          </li>
        </ol>

        <button v-if="Latest.Status === 'approved'" type="button" class="Jump" @click="Open(Latest)">
          Zagłosuj w rankingu
        </button>
      </article>

      <TransitionGroup v-if="Older.length" tag="ul" name="rise" class="Panel Glass">
        <li v-for="Song in Older" :key="Song.Id">
          <component
            :is="Song.Status === 'approved' ? 'button' : 'div'"
            :type="Song.Status === 'approved' ? 'button' : undefined"
            class="Row"
            :class="{ Linked: Song.Status === 'approved' }"
            @click="Open(Song)"
          >
            <Cover :Src="Song.Cover" :Title="Song.Title" :Size="52" :Radius="11" />
            <span class="Text">
              <span class="Name">
                <span class="Clip">{{ Song.Title }}</span>
                <Explicit v-if="Song.Explicit" />
              </span>
              <span class="By">
                <span class="Clip">{{ Song.Artist }}</span>
                <span class="When">{{ When(Song.Created) }}</span>
              </span>
            </span>
            <span class="Mini" :class="Song.Status">
              <span class="MiniLabel">{{ _Short[Song.Status] }}</span>
              <span class="Bars" aria-hidden="true">
                <span v-for="(Seg, Index) in Segments(Song.Status)" :key="Index" class="Bar" :class="Seg" />
              </span>
            </span>
          </component>
        </li>
      </TransitionGroup>
    </div>

    <p v-if="Device && !Vacant" class="Device">
      Identyfikator urządzenia <code>{{ Device.Handle }}</code>
    </p>
  </section>
</template>

<style scoped>
.View {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.Heading {
  font-family: var(--font-display);
  font-size: 34px;
  font-weight: 600;
  line-height: 1.05;
  letter-spacing: -0.035em;
}

.Layout {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.Latest {
  position: relative;
  isolation: isolate;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 18px;
  border-radius: 26px;
  padding: 12px 12px 18px;
}

.Latest::after {
  content: "";
  position: absolute;
  inset: 0;
  z-index: -1;
  background: linear-gradient(180deg, transparent 30%, rgba(12, 11, 14, 0.55));
  pointer-events: none;
}

.Top {
  display: flex;
  align-items: center;
  gap: 16px;
}

.Art {
  position: relative;
  display: block;
  height: 104px;
  width: 104px;
  flex-shrink: 0;
  border-radius: 18px;
  box-shadow: 0 18px 40px -18px rgba(0, 0, 0, 0.9);
}

.Info {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 4px;
}

.LatestName {
  display: -webkit-box;
  overflow: hidden;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: 600;
  line-height: 1.15;
  letter-spacing: -0.025em;
}

.LatestName :deep(.Explicit) {
  margin-left: 6px;
  vertical-align: 2px;
}

.LatestBy {
  overflow: hidden;
  font-size: 14px;
  color: var(--color-ink-mute);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.LatestWhen {
  font-size: 12px;
  color: var(--color-ink-faint);
}

.Track {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin: 0;
  padding: 4px 0 0;
  list-style: none;
}

.Step {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  text-align: center;
}

.Step + .Step::before {
  content: "";
  position: absolute;
  top: 11px;
  right: calc(50% + 17px);
  left: calc(-50% + 17px);
  height: 2px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.08);
}

.Step.done + .Step.done::before,
.Step.done + .Step.waiting::before {
  background: linear-gradient(90deg, var(--color-rose), rgba(224, 168, 168, 0.55));
}

.Step.done + .Step.failed::before {
  background: linear-gradient(90deg, var(--color-rose), rgba(224, 120, 120, 0.4));
}

.Node {
  position: relative;
  z-index: 1;
  display: flex;
  height: 24px;
  width: 24px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 0 0 1.5px rgba(255, 255, 255, 0.12);
}

.Node svg {
  height: 14px;
  width: 14px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.done .Node {
  background: var(--color-rose);
  box-shadow: 0 0 0 4px rgba(224, 168, 168, 0.12);
  color: #2a1216;
}

.waiting .Node {
  background: transparent;
  box-shadow:
    inset 0 0 0 2px var(--color-rose),
    0 0 0 4px rgba(224, 168, 168, 0.1);
}

.waiting .Node::after {
  content: "";
  height: 8px;
  width: 8px;
  border-radius: 999px;
  background: var(--color-rose);
}

.failed .Node {
  background: rgba(224, 108, 108, 0.16);
  box-shadow: inset 0 0 0 1.5px rgba(224, 120, 120, 0.55);
  color: #f0a0a0;
}

.StepLabel {
  font-size: 12.5px;
  font-weight: 500;
  line-height: 1.3;
  color: var(--color-ink-faint);
}

.done .StepLabel {
  color: var(--color-ink-soft);
}

.waiting .StepLabel {
  color: var(--color-rose);
}

.failed .StepLabel {
  color: #e8a0a0;
}

.Jump {
  min-height: 46px;
  border-radius: 999px;
  background: var(--color-ink);
  font-size: 14px;
  font-weight: 600;
  color: #0b0a0d;
  transition: transform 180ms var(--ease-smooth);
}

.Jump:active {
  transform: scale(0.98);
}

.Panel {
  margin: 0;
  border-radius: 24px;
  padding: 4px 0;
  list-style: none;
}

.Panel > li {
  position: relative;
}

.Panel > li + li::before {
  content: "";
  position: absolute;
  top: 0;
  right: 16px;
  left: 78px;
  height: 1px;
  background: rgba(255, 255, 255, 0.05);
}

.Row {
  display: flex;
  width: 100%;
  min-height: 72px;
  align-items: center;
  gap: 14px;
  padding: 10px 14px 10px 12px;
  text-align: left;
}

.Linked {
  transition: background 160ms ease;
}

.Linked:hover {
  background: rgba(255, 255, 255, 0.035);
}

.Panel > li:first-child .Row {
  border-radius: 20px 20px 0 0;
}

.Panel > li:last-child .Row {
  border-radius: 0 0 20px 20px;
}

.Text {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 3px;
}

.Name,
.By {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 6px;
}

.Name {
  font-size: 16px;
  font-weight: 500;
  letter-spacing: -0.01em;
}

.By {
  font-size: 14px;
  color: var(--color-ink-mute);
}

.When {
  flex-shrink: 0;
  color: var(--color-ink-faint);
}

.When::before {
  content: "·";
  margin-right: 6px;
}

.Clip {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Mini {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
  align-items: flex-end;
  gap: 7px;
}

.MiniLabel {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-ink-soft);
}

.Mini.approved .MiniLabel {
  color: var(--color-rose);
}

.Mini.added .MiniLabel {
  color: #a9d8b8;
}

.Mini.rejected .MiniLabel {
  color: var(--color-ink-faint);
}

.Bars {
  display: flex;
  gap: 3px;
}

.Bar {
  height: 4px;
  width: 14px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.1);
}

.Bar.done {
  background: var(--color-rose);
}

.Mini.added .Bar.done {
  background: #a9d8b8;
}

.Bar.failed {
  background: rgba(224, 120, 120, 0.5);
}

.Mini.rejected .Bar.done {
  background: rgba(255, 255, 255, 0.28);
}

.Device {
  margin-top: 6px;
  text-align: center;
  font-size: 12px;
  color: var(--color-ink-faint);
}

.Device code {
  margin-left: 4px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  letter-spacing: 0.03em;
  color: var(--color-ink-mute);
  user-select: all;
}

.GhostLine {
  display: block;
  height: 12px;
  border-radius: 6px;
}

.GhostLine.Short {
  height: 10px;
  width: 40%;
}

@media (min-width: 900px) {
  .View {
    gap: 28px;
  }

  .Heading {
    font-size: 44px;
  }

  .Layout {
    display: grid;
    grid-template-columns: 360px minmax(0, 1fr);
    align-items: start;
    gap: 28px;
  }

  .Latest {
    position: sticky;
    top: calc(var(--head) + 16px);
    gap: 22px;
    border-radius: 30px;
    padding: 14px 14px 22px;
  }

  .Top {
    flex-direction: column;
    align-items: stretch;
    gap: 20px;
  }

  .Art {
    height: auto;
    width: 100%;
    aspect-ratio: 1;
    border-radius: 20px;
  }

  .Info {
    padding: 0 8px;
    gap: 6px;
  }

  .LatestName {
    font-size: 28px;
    line-height: 1.1;
  }

  .LatestBy {
    font-size: 16px;
  }

  .LatestWhen {
    font-size: 13px;
  }

  .Jump {
    margin: 0 8px;
  }

  .Panel {
    border-radius: 28px;
    padding: 6px 0;
  }

  .Row {
    min-height: 80px;
    padding: 12px 20px 12px 16px;
  }

  .Panel > li:first-child .Row {
    border-radius: 24px 24px 0 0;
  }

  .Panel > li:last-child .Row {
    border-radius: 0 0 24px 24px;
  }

  .Panel > li + li::before {
    left: 82px;
    right: 22px;
  }

  .Name {
    font-size: 17px;
  }

  .By {
    font-size: 15px;
  }

  .Bar {
    width: 18px;
  }

  .Device {
    text-align: left;
  }
}

.rise-enter-active {
  transition: opacity 420ms ease, transform 520ms var(--ease-smooth);
}

.rise-enter-from {
  opacity: 0;
  transform: translateY(10px);
}
</style>
