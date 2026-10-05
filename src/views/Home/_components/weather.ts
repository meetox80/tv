export interface WeatherConfig {
  Enabled: boolean
  Latitude: number
  Longitude: number
}

export interface WeatherReading {
  Temperature: number | null
  Condition: string | null
}

export function Describe(Code: number): string | null {
  if (Code === 0) return "Bezchmurnie"
  if (Code === 1) return "Prawie bezchmurnie"
  if (Code === 2) return "Częściowe zachmurzenie"
  if (Code === 3) return "Pochmurno"
  if (Code === 45 || Code === 48) return "Mgła"
  if (Code === 56 || Code === 57) return "Marznąca mżawka"
  if (Code >= 51 && Code <= 55) return "Mżawka"
  if (Code === 66 || Code === 67) return "Marznący deszcz"
  if (Code >= 61 && Code <= 65) return "Deszcz"
  if (Code === 77) return "Śnieg ziarnisty"
  if (Code >= 71 && Code <= 75) return "Śnieg"
  if (Code >= 80 && Code <= 82) return "Przelotny deszcz"
  if (Code === 85 || Code === 86) return "Przelotny śnieg"
  if (Code === 95) return "Burza"
  if (Code === 96 || Code === 99) return "Burza z gradem"
  return null
}

export async function ReadWeather(Config: WeatherConfig | null): Promise<WeatherReading | null> {
  if (!Config?.Enabled) return { Temperature: null, Condition: null }

  const Url = "https://api.open-meteo.com/v1/forecast"
    + `?latitude=${Config.Latitude}&longitude=${Config.Longitude}`
    + "&current=temperature_2m,weather_code&timezone=auto"

  try {
    const Res = await fetch(Url)
    if (!Res.ok) return null

    const Data = await Res.json()
    const Reading = Data?.current?.temperature_2m
    const Code = Data?.current?.weather_code

    return {
      Temperature: typeof Reading === "number" ? Math.round(Reading) : null,
      Condition: typeof Code === "number" ? Describe(Code) : null,
    }
  } catch {
    return null
  }
}
