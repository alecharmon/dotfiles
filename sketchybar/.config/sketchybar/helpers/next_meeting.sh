#!/usr/bin/env bash
# Returns the next calendar event within 6 hours as JSON-like key=value pairs
# Used by sketchybar's meeting widget

SIX_HOURS_LATER=$(date -v+6H "+%Y-%m-%d %H:%M:%S")
NOW_TS=$(date +%s)

# Get next upcoming event: title + datetime + location + notes
EVENT=$(icalBuddy -n -li 1 -ea -nc -nrd -df "%Y-%m-%d" -tf "%H:%M" \
  -iep "title,datetime,location,notes" -b "" -ps "|\n" \
  eventsFrom:today to:today+1 2>/dev/null)

if [ -z "$EVENT" ]; then
  echo "found=no"
  exit 0
fi

TITLE=$(echo "$EVENT" | head -1 | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//')
TIME_LINE=$(echo "$EVENT" | grep -E '^[[:space:]]*[0-9]{4}-[0-9]{2}-[0-9]{2}|^[[:space:]]*[0-9]{2}:[0-9]{2}' | head -1 | sed 's/^[[:space:]]*//')
LOCATION=$(echo "$EVENT" | grep -v "^$" | grep -v "^${TITLE}$" | grep -v "^[[:space:]]*[0-9]" | head -1 | sed 's/^[[:space:]]*//' | sed 's/[[:space:]]*$//' | sed 's/^location: //')

# Parse start time from the time line (format: "HH:MM - HH:MM" or "YYYY-MM-DD HH:MM - HH:MM")
START_TIME=$(echo "$TIME_LINE" | grep -oE '[0-9]{2}:[0-9]{2}' | head -1)
END_TIME=$(echo "$TIME_LINE" | grep -oE '[0-9]{2}:[0-9]{2}' | tail -1)

if [ -z "$START_TIME" ]; then
  echo "found=no"
  exit 0
fi

# Check if the event starts within 6 hours
TODAY=$(date "+%Y-%m-%d")
EVENT_TS=$(date -jf "%Y-%m-%d %H:%M" "$TODAY $START_TIME" "+%s" 2>/dev/null)

if [ -z "$EVENT_TS" ]; then
  echo "found=no"
  exit 0
fi

# Skip events that already ended
if [ -n "$END_TIME" ]; then
  END_TS=$(date -jf "%Y-%m-%d %H:%M" "$TODAY $END_TIME" "+%s" 2>/dev/null)
  if [ -n "$END_TS" ] && [ "$END_TS" -lt "$NOW_TS" ]; then
    echo "found=no"
    exit 0
  fi
fi

SIX_HOURS_TS=$(date -v+6H "+%s")

if [ "$EVENT_TS" -gt "$SIX_HOURS_TS" ]; then
  echo "found=no"
  exit 0
fi

# Extract meeting URL from notes (Google Meet, Zoom, Teams, Webex)
NOTES=$(icalBuddy -n -li 1 -ea -nc -nrd -df "" -tf "" \
  -iep "notes" -b "" -npn \
  eventsFrom:today to:today+1 2>/dev/null)

MEET_URL=""
if [ -n "$NOTES" ]; then
  # Try common video meeting patterns in priority order
  MEET_URL=$(echo "$NOTES" | grep -oE 'https://meet\.google\.com/[a-z]+-[a-z]+-[a-z]+' | head -1)
  if [ -z "$MEET_URL" ]; then
    MEET_URL=$(echo "$NOTES" | grep -oE 'https://[a-z0-9]+\.zoom\.us/j/[0-9]+[^ )*]*' | head -1)
  fi
  if [ -z "$MEET_URL" ]; then
    MEET_URL=$(echo "$NOTES" | grep -oE 'https://teams\.microsoft\.com/l/meetup-join/[^ )*]+' | head -1)
  fi
  if [ -z "$MEET_URL" ]; then
    MEET_URL=$(echo "$NOTES" | grep -oE 'https://[a-z0-9]+\.webex\.com/[^ )*]+' | head -1)
  fi
fi

echo "found=yes"
echo "title=$TITLE"
echo "start=$START_TIME"
echo "end=$END_TIME"
echo "location=$LOCATION"
echo "meet_url=$MEET_URL"
