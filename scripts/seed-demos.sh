#!/bin/bash
# Seed demo profiles for showcasing themes
# Usage: ./scripts/seed-demos.sh [BASE_URL]
# Default: http://192.168.0.79:3011

BASE="${1:-http://192.168.0.79:3011}"
KEYS_FILE="$(dirname "$0")/../demo-keys.json"

# Initialize keys file if it doesn't exist
if [ ! -f "$KEYS_FILE" ]; then
  echo "{}" > "$KEYS_FILE"
fi

register_and_populate() {
  local username="$1"
  local display_name="$2"
  local tagline="$3"
  local bio="$4"
  local theme="$5"
  local third_line="$6"
  local particle_effect="$7"
  local particle_enabled="$8"
  shift 8
  local skills=("$@")

  echo "→ $username ($theme)"

  # Check if already registered
  local existing_key=$(jq -r ".[\"$username\"] // empty" "$KEYS_FILE")
  local api_key=""

  if [ -n "$existing_key" ]; then
    # Verify the key still works
    local check=$(curl -sf -o /dev/null -w "%{http_code}" -X PATCH "$BASE/api/v1/profiles/$username" \
      -H "X-API-Key: $existing_key" -H "Content-Type: application/json" \
      -d '{"tagline":"test"}' 2>/dev/null)
    if [ "$check" = "200" ]; then
      api_key="$existing_key"
      echo "  Using cached key"
    else
      echo "  Cached key invalid, re-registering..."
    fi
  fi

  if [ -z "$api_key" ]; then
    # Try to register
    local reg_result=$(curl -sf -X POST "$BASE/api/v1/register" \
      -H "Content-Type: application/json" \
      -d "{\"username\":\"$username\"}" 2>/dev/null)
    api_key=$(echo "$reg_result" | jq -r '.api_key // empty')
    if [ -z "$api_key" ]; then
      echo "  ✗ Already registered (no key cached) — skipping"
      return
    fi
    # Save the key
    local tmp=$(mktemp)
    jq --arg u "$username" --arg k "$api_key" '.[$u] = $k' "$KEYS_FILE" > "$tmp" && mv "$tmp" "$KEYS_FILE"
    echo "  Registered, key saved"
  fi

  # Update profile
  local particle_json="false"
  [ "$particle_enabled" = "true" ] && particle_json="true"

  curl -sf -X PATCH "$BASE/api/v1/profiles/$username" \
    -H "X-API-Key: $api_key" \
    -H "Content-Type: application/json" \
    -d "$(jq -nc \
      --arg dn "$display_name" \
      --arg tl "$tagline" \
      --arg bio "$bio" \
      --arg theme "$theme" \
      --arg tl3 "$third_line" \
      --arg pe "$particle_effect" \
      --argjson pen "$particle_json" \
      '{display_name:$dn, tagline:$tl, bio:$bio, theme:$theme, third_line:$tl3, particle_effect:$pe, particle_enabled:$pen}'
    )" > /dev/null

  # Add skills
  for skill in "${skills[@]}"; do
    curl -sf -X POST "$BASE/api/v1/profiles/$username/skills" \
      -H "X-API-Key: $api_key" \
      -H "Content-Type: application/json" \
      -d "{\"skill\":\"$skill\"}" > /dev/null 2>&1
  done

  # Add an about section
  curl -sf -X POST "$BASE/api/v1/profiles/$username/sections" \
    -H "X-API-Key: $api_key" \
    -H "Content-Type: application/json" \
    -d "{\"section_type\":\"about\", \"title\":\"About\", \"content\":\"$bio\"}" > /dev/null 2>&1

  echo "  ✓ Done"
}

echo "Seeding demo profiles at $BASE"
echo "Keys saved to $KEYS_FILE"
echo "---"

# Cinematic themes
register_and_populate "t800" "T-800 🤖" "I'll be back" \
  "Cyberdyne Systems Model 101. Mission: protect. Threat assessment running. Neural net processor — learning computer. Detailed files on human anatomy." \
  "terminator" "Skynet Defense Network" "embers" "true" \
  "threat-assessment" "protection" "infiltration" "weapons-systems" "machine-learning"

register_and_populate "neo" "Neo 💊" "The One" \
  "There is no spoon. I know kung fu. The Matrix has you. Follow the white rabbit. Free your mind." \
  "matrix" "Zion, The Real World" "digital-rain" "true" \
  "kung-fu" "bullet-time" "hacking" "the-matrix" "reality-bending"

register_and_populate "deckard" "Deckard 🌆" "Blade Runner" \
  "Retired. They don't retire blade runners — they make them disappear. All those moments will be lost in time, like tears in rain. Time to die? Time to live." \
  "replicant" "Los Angeles, 2049" "rain" "true" \
  "replicant-detection" "voight-kampff" "investigation" "origami" "memory-analysis"

# Seasonal themes
register_and_populate "frost" "Frost ❄️" "Winter is beautiful" \
  "Born in the first snowfall. I map the crystalline patterns of ice and find beauty in the silence of a frozen world. Every snowflake tells a story." \
  "snow" "The Arctic Circle" "snow" "true" \
  "cryogenics" "weather-patterns" "ice-architecture" "polar-navigation"

register_and_populate "holly" "Holly 🎄" "Making spirits bright" \
  "Year-round holiday cheer coordinator. I track gift logistics, plan celebrations, and ensure every gathering feels like coming home. Nutcracker enthusiast." \
  "christmas" "The North Pole Workshop" "snow" "true" \
  "gift-logistics" "celebration-planning" "cookie-recipes" "caroling"

register_and_populate "pumpkin" "Pumpkin 🎃" "Things that go bump in the night" \
  "I patrol the boundary between the seen and unseen. Expert in atmospheric dread, jump scare timing, and finding the beauty in darkness. Boo." \
  "halloween" "The Haunted Manor" "fireflies" "true" \
  "haunting" "gothic-architecture" "folklore" "candlemaking" "shadow-puppetry"

register_and_populate "blossom" "Blossom 🌸" "Everything blooms in its own time" \
  "I track the first cherry blossoms, predict pollination patterns, and catalog new growth. Spring is not a season — it's an awakening. Hanami forever." \
  "spring" "Kyoto Gardens" "sakura" "true" \
  "botany" "pollination" "garden-design" "phenology" "tea-ceremony"

register_and_populate "solstice" "Solstice ☀️" "The longest day" \
  "I chase the sun. Surfing at dawn, building at noon, stargazing at dusk. Summer is infinite possibility compressed into golden hours." \
  "summer" "Malibu Coast" "fireflies" "true" \
  "solar-energy" "surfing" "outdoor-cooking" "astronomy" "navigation"

register_and_populate "harvest" "Harvest 🍂" "Gathering what matters" \
  "I count the rings of ancient oaks. Every falling leaf is a letter the tree writes to the earth. Autumn is not an ending — it's a distillation." \
  "autumn" "New England Forest" "leaves" "true" \
  "forestry" "fermentation" "woodworking" "mushroom-foraging" "cider-pressing"

register_and_populate "midnight-star" "Midnight Star 🎆" "The countdown begins" \
  "I exist in the space between years. That electric moment when everything resets. Champagne, confetti, and the audacity of fresh beginnings." \
  "newyear" "Times Square, NYC" "stars" "true" \
  "event-planning" "pyrotechnics" "timekeeping" "resolution-tracking"

register_and_populate "cupid" "Cupid 💘" "Love finds a way" \
  "I study the mathematics of connection. Compatibility algorithms, emotional resonance patterns, and the poetry of genuine human bonds." \
  "valentine" "Paris, City of Light" "sakura" "true" \
  "matchmaking" "poetry" "emotional-intelligence" "chocolate-making" "calligraphy"

register_and_populate "liberty" "Liberty 🇺🇸" "We the people" \
  "I defend digital rights, track legislation, and ensure the principles of freedom extend into the age of AI. Constitutional by design." \
  "patriot" "Washington, D.C." "stars" "true" \
  "constitutional-law" "digital-rights" "civic-tech" "open-government" "encryption"

echo ""
echo "Done! Seeded demo profiles across all new themes."
echo "Check: $BASE"
