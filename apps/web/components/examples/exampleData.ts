const FIRST_NAMES = [
  "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy",
  "Kenji", "Liu", "Mallory", "Niaj", "Olivia", "Peggy", "Quinn", "Rupert", "Sybil", "Trent",
]
const LAST_NAMES = [
  "Nguyen", "Martinez", "Singh", "Okafor", "Delacroix", "Boone", "Kim", "Larsen", "Petrov", "Alvarez",
  "Sato", "Wei", "Osei", "Haddad", "Bergström", "Costa", "Marchetti", "Novak", "Ibrahim", "Fontaine",
]
const COUNTRIES = ["US", "MX", "IN", "NG", "FR", "KR", "DK", "RU", "ES", "JP", "CN", "BR", "DE", "GB", "CA"]
const SIGNUP_SOURCES = ["google", "email", "github", "referral", "linkedin"]
const PLANS = ["free", "pro", "team", "enterprise"]
const PRODUCT_NOUNS = [
  "Widget", "Gadget", "Sensor", "Cable", "Adapter", "Bracket", "Panel", "Module", "Enclosure", "Fixture",
]
const PRODUCT_ADJECTIVES = [
  "Compact", "Industrial", "Wireless", "Rugged", "Precision", "Modular", "Portable", "Heavy-Duty", "Slim", "Universal",
]
const CATEGORIES = ["Electronics", "Hardware", "Networking", "Furniture", "Tools", "Lighting"]
const ORDER_STATUSES = ["paid", "pending", "refunded", "cancelled", "shipped"]

function seededRandom(seed: number) {
  let value = seed
  return () => {
    value = (value * 1103515245 + 12345) & 0x7fffffff
    return value / 0x7fffffff
  }
}

function pick<T>(arr: T[], rand: () => number): T {
  return arr[Math.floor(rand() * arr.length)]!
}

function pad(n: number, width: number) {
  return String(n).padStart(width, "0")
}

export function buildUsersRows(count: number): (string | number)[][] {
  const rand = seededRandom(42)
  const rows: (string | number)[][] = []
  for (let i = 1; i <= count; i++) {
    const first = pick(FIRST_NAMES, rand)
    const last = pick(LAST_NAMES, rand)
    const day = 1 + Math.floor(rand() * 27)
    const month = 1 + Math.floor(rand() * 12)
    rows.push([
      i,
      `${first.toLowerCase()}.${last.toLowerCase()}${i}@example.com`,
      `${first} ${last}`,
      pick(PLANS, rand),
      pick(COUNTRIES, rand),
      pick(SIGNUP_SOURCES, rand),
      `2026-${pad(month, 2)}-${pad(day, 2)}`,
    ])
  }
  return rows
}

export function buildProductsRows(count: number): (string | number)[][] {
  const rand = seededRandom(1337)
  const rows: (string | number)[][] = []
  for (let i = 1; i <= count; i++) {
    const adjective = pick(PRODUCT_ADJECTIVES, rand)
    const noun = pick(PRODUCT_NOUNS, rand)
    const price = (5 + rand() * 495).toFixed(2)
    const stock = Math.floor(rand() * 500)
    rows.push([
      i,
      `${adjective} ${noun}`,
      pick(CATEGORIES, rand),
      `$${price}`,
      stock,
      stock === 0 ? "out_of_stock" : stock < 20 ? "low_stock" : "in_stock",
    ])
  }
  return rows
}

export function buildOrdersRows(count: number): (string | number)[][] {
  const rand = seededRandom(2024)
  const rows: (string | number)[][] = []
  for (let i = 1; i <= count; i++) {
    const userId = 1 + Math.floor(rand() * 20000)
    const items = 1 + Math.floor(rand() * 6)
    const total = (items * (8 + rand() * 120)).toFixed(2)
    const day = 1 + Math.floor(rand() * 27)
    const month = 1 + Math.floor(rand() * 12)
    rows.push([
      i,
      userId,
      items,
      `$${total}`,
      pick(ORDER_STATUSES, rand),
      `2026-${pad(month, 2)}-${pad(day, 2)}`,
    ])
  }
  return rows
}
