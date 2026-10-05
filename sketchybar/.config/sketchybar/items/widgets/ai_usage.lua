local colors = require("colors")
local settings = require("settings")

local script = os.getenv("HOME") .. "/.config/sketchybar/helpers/ai_usage.py"

-- share of the moon each provider is worth; unknown providers drop out and the rest renormalize
local providers = {
    { key = "claude", name = "Claude Code", weight = 0.45 },
    { key = "codex", name = "Codex", weight = 0.45 },
    { key = "opencode", name = "OpenCode Go", weight = 0.10 },
}

local moon = sbar.add("item", "widgets.ai_usage", {
    position = "right",
    update_freq = 300, -- claude usage endpoint 429s faster polling (retry-after ~210s)
    icon = {
        string = "🌑",
        font = { family = "Apple Color Emoji", size = 14.0 },
        padding_left = 8,
        padding_right = 8
    },
    label = { drawing = false },
    popup = { align = "center" }
})

local rows = {}
for _, p in ipairs(providers) do
    rows[p.key] = sbar.add("item", "widgets.ai_usage." .. p.key, {
        position = "popup." .. moon.name,
        icon = { string = p.name, width = 120, align = "left" },
        label = { string = "??", width = 230, align = "right" }
    })
end

local function left_color(pct)
    if pct == nil then return colors.grey end
    if pct < 15 then return colors.red end
    if pct < 35 then return colors.orange end
    return colors.white
end

-- one lunar cycle over usage: fresh = new moon, half used = full, exhausted = dark again
local phases = { "🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘" }
local function phase(pct_left)
    return phases[math.floor((100 - pct_left) / 100 * 8 + 0.5) % 8 + 1]
end

local function update()
    sbar.exec(script, function(out)
        local left = {}
        for key, s_pct, s_reset, w_pct, w_reset in out:gmatch("(%a+) (%S+) (%S+) (%S+) (%S+)") do
            local s_n, w_n = tonumber(s_pct), tonumber(w_pct)
            left[key] = w_n -- weekly drives the moon
            local function fmt(n, pct, reset)
                return n and (pct .. "% " .. reset) or "—"
            end
            if rows[key] then
                rows[key]:set({
                    label = {
                        string = "5h " .. fmt(s_n, s_pct, s_reset) .. "  wk " .. fmt(w_n, w_pct, w_reset),
                        color = left_color(math.min(s_n or 100, w_n or 100))
                    }
                })
            end
        end

        local sum, weight = 0, 0
        for _, p in ipairs(providers) do
            if left[p.key] then
                sum = sum + left[p.key] * p.weight
                weight = weight + p.weight
            end
        end
        moon:set({ icon = { string = weight > 0 and phase(sum / weight) or "?" } })
    end)
end

moon:subscribe({ "routine", "forced", "system_woke" }, update)

moon:subscribe("mouse.clicked", function()
    moon:set({ popup = { drawing = "toggle" } })
    update()
end)

moon:subscribe("mouse.exited.global", function()
    moon:set({ popup = { drawing = false } })
end)

sbar.add("bracket", "widgets.ai_usage.bracket", { moon.name }, {
    background = {
        color = colors.bg1,
        border_color = colors.grey,
        border_width = 1
    }
})

sbar.add("item", "widgets.ai_usage.padding", {
    position = "right",
    width = settings.group_paddings
})

update()
