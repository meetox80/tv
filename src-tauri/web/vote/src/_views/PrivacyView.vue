<script setup lang="ts">
const Emit = defineEmits<{ Back: [] }>()
</script>

<template>
  <article class="Policy" aria-labelledby="policy-heading">
    <button type="button" class="Back" @click="Emit('Back')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M20 12H4" />
        <path d="m10 18-6-6 6-6" />
      </svg>
      Wróć
    </button>

    <header class="Head">
      <h1 id="policy-heading" class="Heading">Polityka prywatności</h1>
      <p class="Updated">Aktualizacja: 27 września 2026</p>
    </header>

    <section class="Block">
      <h2>1. Administrator danych</h2>
      <p>
        Administratorem danych osobowych jest autor zstio-tv, szkolnego projektu realizowanego w ZSTiO.
        W sprawach dotyczących danych osobowych możesz skontaktować się z nim przez Telegram lub e-mail.
      </p>
      <p class="Contact">
        <span>Telegram: <a href="https://t.me/kotwicha" target="_blank" rel="noopener noreferrer">t.me/kotwicha</a></span>
        <span>E-mail: <code>ox80@mail.i2p</code></span>
      </p>
    </section>

    <section class="Block">
      <h2>2. Jakie dane przetwarzamy</h2>
      <p>
        Korzystanie z głosowania nie wymaga zakładania konta ani podawania imienia, adresu e-mail czy
        numeru telefonu. Przetwarzamy wyłącznie losowy identyfikator urządzenia zapisany w pliku cookie,
        zgłoszone propozycje utworów, oddane głosy oraz datę ostatniej wizyty.
      </p>
      <p>
        Adres IP wykorzystujemy tylko do ochrony przed botami i nie zapisujemy go.
      </p>
    </section>

    <section class="Block">
      <h2>3. Cel i podstawa prawna</h2>
      <p>
        Dane przetwarzamy, aby każde urządzenie mogło zgłosić określoną liczbę propozycji dziennie
        i oddać jeden głos na każdy utwór, a także aby zapobiegać nadużyciom, w tym blokować urządzenia
        naruszające zasady głosowania.
      </p>
      <p>
        Podstawą prawną jest prawnie uzasadniony interes administratora, polegający na zapewnieniu
        rzetelnego przebiegu głosowania (art. 6 ust. 1 lit. f RODO). Plik cookie jest niezbędny do
        działania głosowania, dlatego nie wymaga zgody.
      </p>
    </section>

    <section class="Block">
      <h2>4. Okres przechowywania</h2>
      <p>
        Identyfikator urządzenia i oddane głosy usuwamy po 180 dniach nieaktywności. Zgłoszone
        propozycje pozostają w serwisie, ale nie są już powiązane z żadnym urządzeniem. Dane urządzenia
        zablokowanego przechowujemy przez cały czas trwania blokady.
      </p>
    </section>

    <section class="Block">
      <h2>5. Usługi zewnętrzne</h2>
      <p>
        Serwis działa za pośrednictwem Cloudflare, który obsługuje połączenie ze stroną i sprawdza, czy
        użytkownik nie jest botem. W tym celu Cloudflare przetwarza m.in. adres IP i informacje
        o przeglądarce zgodnie ze swoją
        <a href="https://www.cloudflare.com/privacypolicy/" target="_blank" rel="noopener noreferrer">polityką prywatności</a>.
      </p>
      <p>Nie korzystamy z narzędzi analitycznych ani reklamowych.</p>
    </section>

    <section class="Block">
      <h2>6. Twoje prawa</h2>
      <p>
        Masz prawo dostępu do swoich danych, ich usunięcia, ograniczenia przetwarzania oraz wniesienia
        sprzeciwu wobec przetwarzania. Aby skorzystać z tych praw, skontaktuj się z administratorem
        i podaj identyfikator urządzenia widoczny w zakładce „Twoje”. Bez niego nie możemy ustalić,
        które dane dotyczą Ciebie.
      </p>
      <p>
        Przysługuje Ci również prawo wniesienia skargi do Prezesa Urzędu Ochrony Danych Osobowych.
      </p>
    </section>

    <section class="Block">
      <h2>7. Zmiany polityki prywatności</h2>
      <p>
        O zmianach informujemy, aktualizując tę stronę oraz datę widoczną na jej początku.
      </p>
    </section>
  </article>
</template>

<style scoped>
.Policy {
  display: flex;
  flex-direction: column;
  gap: 28px;
  padding-bottom: 12px;
}

.Back {
  display: inline-flex;
  min-height: 44px;
  align-self: flex-start;
  align-items: center;
  gap: 8px;
  border-radius: 12px;
  padding: 0 12px 0 6px;
  font-size: 14px;
  color: var(--color-ink-mute);
  transition: color 160ms ease, background 160ms ease;
}

.Back:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-ink);
}

.Back svg {
  height: 18px;
  width: 18px;
}

.Head {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.Heading {
  font-family: var(--font-display);
  font-size: 34px;
  font-weight: 600;
  line-height: 1.05;
  letter-spacing: -0.035em;
}

.Updated {
  font-size: 13px;
  color: var(--color-ink-faint);
}

.Block {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.Block h2 {
  font-family: var(--font-display);
  font-size: 19px;
  font-weight: 600;
  letter-spacing: -0.015em;
}

.Block p {
  font-size: 15px;
  line-height: 1.65;
  color: var(--color-ink-soft);
}

.Block .Contact {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.Block code {
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  padding: 1px 6px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.9em;
  color: var(--color-ink);
  word-break: break-all;
}

.Block a {
  color: var(--color-rose);
  text-decoration: underline;
  text-underline-offset: 3px;
  text-decoration-color: rgba(224, 168, 168, 0.4);
}

.Block a:hover {
  text-decoration-color: currentColor;
}
</style>
