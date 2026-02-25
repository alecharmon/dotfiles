local icons = require("icons")
local colors = require("colors")
local settings = require("settings")

local slack = sbar.add("item", "widgets.slack", {
    position = "right",
    drawing = false,
    icon = {
        string = ":slack:",
        color = colors.white,
        padding_left = 8,
        font = {
            family = settings.icons
        }
    },
    label = {
        color = colors.white,
        padding_right = 8,
        font = {
            family = settings.font.numbers,
            style = settings.font.style_map["Bold"],
            size = 12.0
        }
    },
    padding_left = 1,
    padding_right = 1,
    background = {
        color = colors.bg1,
        border_color = colors.grey,
        border_width = 1
    },
    update_freq = 5
})

local function update_slack()
    sbar.exec("lsappinfo info -only StatusLabel 'Slack' 2>/dev/null", function(output)
        local count = output:match('"label"="(%d+)"')
        if count and tonumber(count) > 0 then
            slack:set({
                drawing = true,
                label = count
            })
        else
            slack:set({ drawing = false })
        end
    end)
end

slack:subscribe({"routine", "forced", "system_woke"}, function(_)
    update_slack()
end)

slack:subscribe("mouse.clicked", function(_)
    sbar.exec("open -a 'Slack'")
end)

-- Padding after slack widget
sbar.add("item", "widgets.slack.padding", {
    position = "right",
    width = settings.group_paddings,
    drawing = false
})

-- Initial fetch
update_slack()
