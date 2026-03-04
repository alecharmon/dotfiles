#!/bin/bash
# Move apps to their default workspaces
aerospace list-windows --all --format '%{app-bundle-id}|%{window-id}' | while IFS='|' read -r app_id window_id; do
    case "$app_id" in
        com.mitchellh.ghostty)
            aerospace move-node-to-workspace --window-id "$window_id" 1 ;;
        com.microsoft.VSCode)
            aerospace move-node-to-workspace --window-id "$window_id" 2 ;;
        company.thebrowser.Browser)
            aerospace move-node-to-workspace --window-id "$window_id" 3 ;;
        com.tinyspeck.slackmacgap)
            aerospace move-node-to-workspace --window-id "$window_id" 4 ;;
        *)
            aerospace move-node-to-workspace --window-id "$window_id" 4 ;;
    esac
done
