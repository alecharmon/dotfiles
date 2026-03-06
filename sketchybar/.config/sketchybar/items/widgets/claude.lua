local colors = require("colors")
local settings = require("settings")

local claude = sbar.add("item", "widgets.claude", {
    position = "right",
    drawing = false,
    icon = {
        string = "󰧑",
        color = colors.yellow,
        padding_left = 8,
        font = {
            family = settings.font.text,
            size = 16.0
        }
    },
    label = {
        string = "Waiting",
        color = colors.yellow,
        padding_right = 8,
        font = {
            family = settings.font.text,
            size = 12.0
        }
    },
    padding_left = 1,
    padding_right = 1,
    background = {
        color = colors.bg1,
        border_color = colors.yellow,
        border_width = 1
    },
    updates = "on",
    update_freq = 5
})

local function update_claude()
    sbar.exec("test -f ~/.local/state/claude-code-waiting && echo 'waiting' || echo 'busy'", function(output)
        local trimmed = output:gsub("%s+", "")
        if trimmed == "waiting" then
            claude:set({ drawing = true })
        else
            claude:set({ drawing = false })
        end
    end)
end

claude:subscribe({"routine", "forced", "system_woke"}, function(_)
    update_claude()
end)

claude:subscribe("mouse.clicked", function(_)
    sbar.exec("rm -f ~/.local/state/claude-code-waiting && open -a 'Ghostty'")
    claude:set({ drawing = false })
end)

-- Padding after claude widget
sbar.add("item", "widgets.claude.padding", {
    position = "right",
    width = settings.group_paddings,
    drawing = false
})

-- Initial check
update_claude()
