local settings = require("settings")
local colors = require("colors")

-- Padding item required because of bracket
sbar.add("item", {
    position = "right",
    width = settings.group_paddings
})

local cal = sbar.add("item", {
    icon = {
        color = colors.white,
        padding_left = 8,
        font = {
            size = 22.0
        }
    },
    label = {
        color = colors.white,
        padding_right = 8,
        width = 80,
        align = "right",
        font = {
            family = settings.icons
        }
    },
    position = "right",
    update_freq = 5,
    padding_left = 1,
    padding_right = 1,
    background = {
        color = colors.bg2,
        border_color = colors.grey,
        border_width = 1
    }
})

-- Double border for calendar using a single item bracket
-- sbar.add("bracket", { cal.name }, {
--   background = {
--     color = colors.transparent,
--     height = 30,
--     border_color = colors.grey,
--   }
-- })

-- Padding item required because of bracket
sbar.add("item", {
    position = "right",
    width = settings.group_paddings
})

local show_utc = false

local function update_time()
    local label, width
    if show_utc then
        label = os.date("!%m/%d %H:%M UTC")
        width = 110
    else
        label = os.date("%m/%d %H:%M")
        width = 80
    end
    cal:set({
        icon = "",
        label = { string = label, width = width }
    })
end

cal:subscribe({"forced", "routine", "system_woke"}, function(env)
    update_time()
end)

cal:subscribe("mouse.clicked", function(env)
    show_utc = not show_utc
    update_time()
end)
