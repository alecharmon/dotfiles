local icons = require("icons")
local colors = require("colors")
local settings = require("settings")

local memory = sbar.add("graph", "widgets.memory", 42, {
    position = "right",
    update_freq = 2,
    graph = {
        color = colors.blue
    },
    background = {
        height = 22,
        color = {
            alpha = 0
        },
        border_color = {
            alpha = 0
        },
        drawing = true
    },
    icon = {
        string = icons.memory
    },
    label = {
        string = "mem ??%",
        font = {
            family = settings.font.numbers,
            style = settings.font.style_map["Bold"],
            size = 9.0
        },
        align = "right",
        padding_right = 0,
        width = 0,
        y_offset = 4
    },
    padding_right = settings.paddings + 6
})

local memory_command = [[
total=$(sysctl -n hw.memsize)
vm_stat | awk -v total="$total" '
/page size of/ { page_size=$8; gsub(/[^0-9]/, "", page_size) }
/Pages free/ { free=$3 }
/Pages speculative/ { speculative=$3 }
/File-backed pages/ { filebacked=$3 }
END {
  gsub(/\./, "", free)
  gsub(/\./, "", speculative)
  gsub(/\./, "", filebacked)
  used = (int(total / page_size) - free - speculative - filebacked) * page_size
  print used
  print total
}'
]]

local function memory_color(load)
    if load > 30 then
        if load < 60 then
            return colors.yellow
        elseif load < 80 then
            return colors.orange
        else
            return colors.red
        end
    end

    return colors.blue
end

memory:subscribe("routine", function()
    sbar.exec(memory_command, function(result)
        local used, total = result:match("(%d+)\n(%d+)")
        used = tonumber(used)
        total = tonumber(total)

        if not used or not total or total == 0 then
            return
        end

        local load = math.floor((used / total * 100) + 0.5)
        memory:push({load / 100.})
        memory:set({
            graph = {
                color = memory_color(load)
            },
            label = "mem " .. load .. "%"
        })
    end)
end)

memory:subscribe("mouse.clicked", function(env)
    sbar.exec("open -a 'Activity Monitor'")
end)

sbar.add("bracket", "widgets.memory.bracket", {memory.name}, {
    background = {
        color = colors.bg1,
        border_color = colors.grey,
        border_width = 1
    }
})

sbar.add("item", "widgets.memory.padding", {
    position = "right",
    width = settings.group_paddings
})
