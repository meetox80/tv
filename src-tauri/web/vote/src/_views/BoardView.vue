<script setup lang="ts">
import { ref, computed, watch, nextTick, onDeactivated, type Ref } from "vue"
import Blank from "../_components/Blank.vue"
import Explicit from "../_components/Explicit.vue"
import PlayCover from "../_components/PlayCover.vue"
import VoteControl from "../_components/VoteControl.vue"
import { vFit } from "../_directives/fit"
import { Describe } from "../_api/messages"
import type { Ballot } from "../_api/types"
import { ActiveTab, Banned, Board, Cancelled, Highlight, Open, SongsReady, Vote } from "../_state/store"
import { Stop } from "../_state/preview"

const Busy: Ref<Set<number>> = ref(new Set())
const Problem: Ref<string> = ref("")
const Flash: Ref<number | null> = ref(null)

const Leader = computed(() => Board.value[0] ?? null)
const Rest = computed(() => Board.value.slice(1))
const Vacant = computed(() => SongsReady.value && !Board.value.length)

let _ProblemTimer: ReturnType<typeof setTimeout> | null = null

function _Say(Message: string): void {
  Problem.value = Message
  if (_ProblemTimer) clearTimeout(_ProblemTimer)
  _ProblemTimer = setTimeout(() => (Problem.value = ""), 5000)
}

async function Cast(Id: number, Wanted: Ballot): Promise<void> {
  if (Busy.value.has(Id)) return
  Busy.value = new Set(Busy.value).add(Id)

  try {
    await Vote(Id, Wanted)
  } catch (Err) {
    if (!(Err instanceof Cancelled)) _Say(Describe(Err))
  } finally {
    const Next = new Set(Busy.value)
    Next.delete(Id)
    Busy.value = Next
  }
}

watch([Highlight, SongsReady, ActiveTab], async () => {
  const Id = Highlight.value
  if (Id === null || ActiveTab.value !== "board" || !SongsReady.value) return

  await nextTick()
  const Row = document.getElementById(`song-${Id}`)
  Highlight.value = null
  if (!Row) return

  Row.scrollIntoView({ behavior: "smooth", block: "center" })
  Flash.value = Id
  setTimeout(() => {
    if (Flash.value === Id) Flash.value = null
  }, 2200)
}, { immediate: true })

onDeactivated(Stop)
</script>

<template>
  <section class="View" aria-labelledby="board-heading">
    <h1 id="board-heading" :class="Vacant ? 'SrOnly' : 'Heading'">Ranking</h1>

    <div aria-live="assertive">
      <Transition name="fade">
        <p v-if="Problem" class="Problem" role="alert">{{ Problem }}</p>
      </Transition>
    </div>

    <div v-if="!SongsReady" class="Layout" aria-hidden="true">
      <div class="Leader Glass">
        <span class="LeaderArt Skeleton" />
        <span class="LeaderText">
          <span class="Skeleton GhostLine" style="width: 70%" />
          <span class="Skeleton GhostLine Short" />
        </span>
      </div>
      <ol class="Panel Glass">
        <li v-for="I in 5" :key="I" class="Row">
          <span class="Skeleton GhostRank" />
          <span class="Skeleton GhostThumb" />
          <span class="Text">
            <span class="Skeleton GhostLine" :style="{ width: `${50 + ((I * 13) % 30)}%` }" />
            <span class="Skeleton GhostLine Short" />
          </span>
        </li>
      </ol>
    </div>

    <Blank v-else-if="Vacant" Title="Ranking jest jeszcze pusty" Text="Zatwierdzone propozycje pojawią się tutaj.">
      <button v-if="Open" type="button" @click="ActiveTab = 'request'">Zaproponuj utwór</button>
    </Blank>

    <div v-else class="Layout">
      <article
        v-if="Leader"
        :id="`song-${Leader.Id}`"
        class="Leader Glass"
        :class="{ Flash: Flash === Leader.Id }"
      >
        <div class="LeaderArt">
          <PlayCover :Id="Leader.Id" :Src="Leader.Cover" :Title="Leader.Title" :Size="140" Fill Large />
          <span class="Badge" aria-hidden="true">1</span>
        </div>

        <div class="LeaderText">
          <p v-fit class="LeaderName">
            <span class="SrOnly">Miejsce 1: </span>
            <span>{{ Leader.Title }}</span>
            <Explicit v-if="Leader.Explicit" />
          </p>
          <p v-fit class="LeaderBy">{{ Leader.Artist }}</p>

          <VoteControl
            Wide
            class="LeaderVote"
            :Value="Leader.Vote"
            :Score="Leader.Score"
            :Title="Leader.Title"
            :Busy="Busy.has(Leader.Id)"
            :ReadOnly="!Open || Banned"
            @Cast="(Wanted: Ballot) => Cast(Leader!.Id, Wanted)"
          />
        </div>
      </article>

      <TransitionGroup v-if="Rest.length" tag="ol" name="board" class="Panel Glass">
        <li
          v-for="(Song, Index) in Rest"
          :id="`song-${Song.Id}`"
          :key="Song.Id"
          class="Row"
          :class="{ Flash: Flash === Song.Id }"
        >
          <span class="Rank" aria-hidden="true">{{ Index + 2 }}</span>
          <PlayCover :Id="Song.Id" :Src="Song.Cover" :Title="Song.Title" :Size="56" :Radius="12" />

          <div class="Text">
            <p class="Name">
              <span class="SrOnly">Miejsce {{ Index + 2 }}: </span>
              <span class="Clip">{{ Song.Title }}</span>
              <Explicit v-if="Song.Explicit" />
            </p>
            <p class="By"><span class="Clip">{{ Song.Artist }}</span></p>
          </div>

          <VoteControl
            :Value="Song.Vote"
            :Score="Song.Score"
            :Title="Song.Title"
            :Busy="Busy.has(Song.Id)"
            :ReadOnly="!Open || Banned"
            @Cast="(Wanted: Ballot) => Cast(Song.Id, Wanted)"
          />
        </li>
      </TransitionGroup>
    </div>
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

.Problem {
  border-radius: 14px;
  background: rgba(224, 108, 108, 0.08);
  box-shadow: inset 0 0 0 1px rgba(224, 108, 108, 0.2);
  padding: 12px 14px;
  font-size: 14px;
  line-height: 1.45;
  color: #f0c4c4;
}

.Layout {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.Leader {
  display: flex;
  align-items: center;
  gap: 16px;
  border-radius: 26px;
  padding: 12px;
  transition: box-shadow 500ms ease;
}

.LeaderArt {
  position: relative;
  height: 116px;
  width: 116px;
  flex-shrink: 0;
  border-radius: 18px;
  box-shadow: 0 18px 40px -18px rgba(0, 0, 0, 0.9);
}

.Badge {
  position: absolute;
  top: 8px;
  left: 8px;
  display: flex;
  height: 26px;
  min-width: 26px;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: var(--color-rose);
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 700;
  color: #1d1113;
  box-shadow: 0 6px 16px -6px rgba(0, 0, 0, 0.7);
}

.LeaderText {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.LeaderName {
  --fit-min: 17px;
  --fit-max: 24px;
  max-width: 100%;
  overflow: hidden;
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: 600;
  line-height: 1.15;
  letter-spacing: -0.025em;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.LeaderName[data-wrap="1"] {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  white-space: normal;
}

.LeaderName :deep(.Explicit) {
  margin-left: 6px;
  vertical-align: 2px;
}

.LeaderBy {
  --fit-min: 13px;
  --fit-max: 15px;
  max-width: 100%;
  overflow: hidden;
  font-size: 14px;
  color: var(--color-ink-mute);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.LeaderVote {
  margin-top: 8px;
}

.Panel {
  position: relative;
  margin: 0;
  border-radius: 24px;
  padding: 4px 0;
  list-style: none;
}

.Row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 8px 6px 12px;
  transition: background 500ms ease;
}

.Row + .Row::before {
  content: "";
  position: absolute;
  top: 0;
  right: 16px;
  left: 102px;
  height: 1px;
  background: rgba(255, 255, 255, 0.05);
}

.Rank {
  width: 22px;
  flex-shrink: 0;
  text-align: center;
  font-family: var(--font-display);
  font-size: 15px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--color-ink-faint);
}

.Text {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 3px;
}

.Name {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 6px;
  font-size: 16px;
  font-weight: 500;
  letter-spacing: -0.01em;
}

.By {
  display: flex;
  min-width: 0;
  font-size: 14px;
  color: var(--color-ink-mute);
}

.Clip {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.Leader.Flash {
  box-shadow:
    inset 0 0 0 1px rgba(224, 168, 168, 0.6),
    0 0 0 6px rgba(224, 168, 168, 0.08);
}

.Row.Flash {
  background: rgba(224, 168, 168, 0.08);
}

.GhostRank {
  height: 14px;
  width: 22px;
  border-radius: 6px;
}

.GhostThumb {
  height: 56px;
  width: 56px;
  flex-shrink: 0;
  border-radius: 12px;
}

.GhostLine {
  display: block;
  height: 12px;
  border-radius: 6px;
}

.GhostLine.Short {
  height: 10px;
  width: 34%;
}

.Leader .GhostLine {
  width: 100%;
}

.Leader .GhostLine.Short {
  width: 40%;
}

.Row :deep(.Arrow) {
  height: 34px;
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

  .Leader {
    position: sticky;
    top: calc(var(--head) + 16px);
    flex-direction: column;
    align-items: stretch;
    gap: 20px;
    border-radius: 30px;
    padding: 14px 14px 22px;
  }

  .LeaderArt {
    height: auto;
    width: 100%;
    aspect-ratio: 1;
    border-radius: 20px;
  }

  .Badge {
    top: 14px;
    left: 14px;
    height: 32px;
    min-width: 32px;
    font-size: 15px;
  }

  .LeaderText {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    grid-template-areas:
      "name vote"
      "by vote";
    align-items: center;
    column-gap: 14px;
    row-gap: 4px;
    padding: 0 4px 0 8px;
  }

  .LeaderName {
    --fit-min: 19px;
    --fit-max: 32px;
    grid-area: name;
    align-self: end;
    line-height: 1.1;
  }

  .LeaderBy {
    --fit-min: 14px;
    --fit-max: 17px;
    grid-area: by;
    align-self: start;
  }

  .LeaderVote {
    grid-area: vote;
    margin-top: 0;
  }

  .LeaderVote :deep(.Arrow) {
    width: 44px;
  }

  .LeaderVote :deep(.Count) {
    min-width: 42px;
  }

  .Panel {
    border-radius: 28px;
    padding: 6px 0;
  }

  .Row {
    gap: 16px;
    padding: 8px 14px 8px 18px;
  }

  .Row + .Row::before {
    left: 116px;
    right: 22px;
  }

  .Rank {
    width: 26px;
    font-size: 17px;
  }

  .Row {
    min-height: 80px;
  }

  .Row :deep(.Vote) {
    width: auto;
    flex-direction: row;
    gap: 2px;
  }

  .Row :deep(.Arrow) {
    height: 40px;
    width: 40px;
    border-radius: 999px;
  }

  .Row :deep(.Count) {
    min-width: 40px;
  }

  .Name {
    font-size: 17px;
  }

  .By {
    font-size: 15px;
  }
}

.board-move {
  transition: transform 620ms var(--ease-smooth);
}

.board-enter-active {
  transition: opacity 420ms ease, transform 520ms var(--ease-smooth);
}

.board-leave-active {
  position: absolute;
  right: 0;
  left: 0;
  transition: opacity 240ms ease;
}

.board-enter-from {
  opacity: 0;
  transform: translateY(12px);
}

.board-leave-to {
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 240ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
