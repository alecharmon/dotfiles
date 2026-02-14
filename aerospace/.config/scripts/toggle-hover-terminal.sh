#!/usr/bin/env bash

HOVER_WS="H"
CURRENT_WS=$(aerospace list-workspaces --focused)

if [ "$CURRENT_WS" = "$HOVER_WS" ]; then
  # Already on hover workspace, toggle back to previous
  aerospace workspace-back-and-forth
else
  # Check if hover-terminal window exists
  WINDOW_ID=$(aerospace list-windows --all | grep hover-terminal | awk '{print $1}')

  if [ -z "$WINDOW_ID" ]; then
    # Launch a new hover-terminal instance
    ghostty --title=hover-terminal &
    disown
  else
    # Switch to the hover workspace
    aerospace workspace "$HOVER_WS"
  fi
fi
