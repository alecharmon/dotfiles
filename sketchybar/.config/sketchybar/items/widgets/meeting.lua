local icons = require("icons")
local colors = require("colors")
local settings = require("settings")

local meeting = sbar.add("item", "widgets.meeting", {
    position = "right",
    drawing = false,
    icon = {
        string = "􀧞",
        color = colors.yellow,
        padding_left = 8
    },
    label = {
        color = colors.white,
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
        border_color = colors.grey,
        border_width = 1
    },
    update_freq = 60,
    popup = {
        align = "right",
        background = {
            color = colors.bg2,
            border_color = colors.grey,
            border_width = 1,
            corner_radius = 8
        }
    }
})

local popup_title = sbar.add("item", {
    position = "popup." .. meeting.name,
    icon = {
        string = "􀧞",
        color = colors.yellow,
        padding_left = 8
    },
    label = {
        color = colors.white,
        padding_right = 8,
        font = {
            family = settings.font.text,
            style = "Bold",
            size = 13.0
        }
    },
    background = { drawing = false }
})

local popup_time = sbar.add("item", {
    position = "popup." .. meeting.name,
    icon = {
        string = "􀐫",
        color = colors.grey,
        padding_left = 8
    },
    label = {
        color = colors.grey,
        padding_right = 8,
        font = {
            family = settings.font.text,
            size = 12.0
        }
    },
    background = { drawing = false }
})

local popup_location = sbar.add("item", {
    position = "popup." .. meeting.name,
    drawing = false,
    icon = {
        string = "􀎫",
        color = colors.grey,
        padding_left = 8
    },
    label = {
        color = colors.grey,
        padding_right = 8,
        font = {
            family = settings.font.text,
            size = 12.0
        }
    },
    background = { drawing = false }
})

local popup_meet_link = sbar.add("item", {
    position = "popup." .. meeting.name,
    drawing = false,
    icon = {
        string = "􀎞",
        color = colors.blue,
        padding_left = 8
    },
    label = {
        color = colors.blue,
        padding_right = 8,
        font = {
            family = settings.font.text,
            size = 12.0
        }
    },
    background = { drawing = false }
})

local current_meet_url = ""

local function parse_meeting_output(output)
    local result = {}
    for line in output:gmatch("([^\n]+)") do
        local key, value = line:match("^(.-)=(.*)$")
        if key and value then
            result[key] = value
        end
    end
    return result
end

local function update_meeting()
    sbar.exec("$CONFIG_DIR/helpers/next_meeting.sh", function(output)
        local data = parse_meeting_output(output)

        if data.found == "yes" then
            local title = data.title or ""
            local start_time = data.start or ""
            local end_time = data.end_time or data["end"] or ""
            local location = data.location or ""
            local meet_url = data.meet_url or ""

            current_meet_url = meet_url

            local display_title = title
            if #display_title > 20 then
                display_title = display_title:sub(1, 20) .. "…"
            end

            meeting:set({
                drawing = true,
                label = display_title .. "  " .. start_time
            })

            popup_title:set({ label = title })

            local time_str = start_time
            if end_time ~= "" then
                time_str = start_time .. " – " .. end_time
            end
            popup_time:set({ label = time_str })

            if location ~= "" then
                popup_location:set({
                    drawing = true,
                    label = location
                })
            else
                popup_location:set({ drawing = false })
            end

            if meet_url ~= "" then
                -- Show a friendly label based on the URL
                local link_label = "Join Meeting"
                if meet_url:find("meet%.google%.com") then
                    link_label = "Join Google Meet"
                elseif meet_url:find("zoom%.us") then
                    link_label = "Join Zoom"
                elseif meet_url:find("teams%.microsoft%.com") then
                    link_label = "Join Teams"
                elseif meet_url:find("webex%.com") then
                    link_label = "Join Webex"
                end
                popup_meet_link:set({
                    drawing = true,
                    label = link_label
                })
            else
                popup_meet_link:set({ drawing = false })
            end
        else
            current_meet_url = ""
            meeting:set({ drawing = false })
            meeting:set({ popup = { drawing = false } })
        end
    end)
end

meeting:subscribe({"routine", "forced", "system_woke"}, function(_)
    update_meeting()
end)

meeting:subscribe("mouse.clicked", function(_)
    meeting:set({ popup = { drawing = "toggle" } })
end)

meeting:subscribe("mouse.exited.global", function(_)
    meeting:set({ popup = { drawing = false } })
end)

popup_meet_link:subscribe("mouse.clicked", function(_)
    if current_meet_url ~= "" then
        sbar.exec("open '" .. current_meet_url .. "'")
        meeting:set({ popup = { drawing = false } })
    end
end)

-- Padding after meeting widget
sbar.add("item", "widgets.meeting.padding", {
    position = "right",
    width = settings.group_paddings,
    drawing = false
})

-- Initial fetch
update_meeting()
