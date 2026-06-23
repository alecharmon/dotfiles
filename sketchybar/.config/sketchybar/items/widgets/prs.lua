local colors = require("colors")
local settings = require("settings")

local script = os.getenv("SKETCHYBAR_PRS_SCRIPT") or (os.getenv("HOME") .. "/dev/dotfiles/scripts/sketchybar-prs")
local max_rows_per_section = 8

local function shell_quote(value)
    return "'" .. tostring(value or ""):gsub("'", "'\\''") .. "'"
end

local function split_lines(value)
    local lines = {}
    for line in tostring(value or ""):gmatch("[^\r\n]+") do
        table.insert(lines, line)
    end
    return lines
end

local function split_tabs(value)
    local fields = {}
    local start = 1
    while true do
        local tab = string.find(value, "\t", start, true)
        if not tab then
            table.insert(fields, string.sub(value, start))
            break
        end
        table.insert(fields, string.sub(value, start, tab - 1))
        start = tab + 1
    end
    return fields
end

local prs = sbar.add("item", {
    position = "right",
    update_freq = 60,
    icon = {
        string = "",
        color = colors.white,
        font = {
            family = settings.font.text,
            style = settings.font.style_map["Bold"],
            size = 15.0
        },
        padding_left = 8,
        padding_right = 4
    },
    label = {
        string = "0",
        color = colors.white,
        padding_left = 2,
        padding_right = 8
    },
    background = {
        color = colors.bg1,
        border_color = colors.grey,
        border_width = 1
    },
    popup = {
        align = "right"
    }
})

local refresh = sbar.add("item", {
    position = "popup." .. prs.name,
    icon = { string = "󰑓", color = colors.blue },
    label = { string = "Refresh PRs", width = 520, align = "left" }
})

local mine_header = sbar.add("item", {
    position = "popup." .. prs.name,
    icon = { drawing = false },
    label = { string = "Mine", color = colors.yellow, width = 520, align = "left" }
})

local mine_rows = {}
for i = 1, max_rows_per_section do
    mine_rows[i] = sbar.add("item", {
        position = "popup." .. prs.name,
        drawing = false,
        icon = { drawing = false },
        label = { width = 520, align = "left", max_chars = 100 },
        click_script = "true"
    })
end

local review_header = sbar.add("item", {
    position = "popup." .. prs.name,
    icon = { drawing = false },
    label = { string = "Review requests", color = colors.yellow, width = 520, align = "left" }
})

local review_rows = {}
for i = 1, max_rows_per_section do
    review_rows[i] = sbar.add("item", {
        position = "popup." .. prs.name,
        drawing = false,
        icon = { drawing = false },
        label = { width = 520, align = "left", max_chars = 100 },
        click_script = "true"
    })
end

local empty_row = sbar.add("item", {
    position = "popup." .. prs.name,
    icon = { drawing = false },
    label = { string = "No PRs", color = colors.with_alpha(colors.white, 0.65), width = 520, align = "left" }
})

local function set_headers(has_mine, has_review)
    mine_header:set({ drawing = has_mine })
    review_header:set({ drawing = has_review })
end

local function popup_visible()
    local ok, query = pcall(function()
        return prs:query()
    end)
    return ok and query and query.popup and query.popup.drawing == "on"
end

local function render_summary()
    sbar.exec(shell_quote(script) .. " render --format summary", function(output)
        local fields = split_tabs(output)
        local count = fields[1] or "0"
        local state = fields[2] or "ok"
        local color = colors.white
        if state == "error" then
            color = colors.red
        elseif state == "stale" then
            color = colors.orange
        end
        prs:set({ label = { string = count, color = color }, icon = { color = color } })
    end)
end

local function set_row(row, display_name, age, status, title, url, is_stack, stack_position)
    local icon = is_stack and "󰓾" or ""
    local label = display_name .. " · " .. age .. " · " .. status .. " · " .. title
    if is_stack and stack_position ~= "" then
        label = stack_position .. " · " .. label
    end
    local click_script = "true"
    if url ~= "" then
        click_script = "open " .. shell_quote(url)
    end
    row:set({
        drawing = true,
        icon = { string = icon, drawing = true, color = is_stack and colors.magenta or colors.white },
        label = { string = label },
        click_script = click_script
    })
end

local function hide_unused(rows, start_index)
    for i = start_index, max_rows_per_section do
        rows[i]:set({ drawing = false, click_script = "true" })
    end
end

local function render_popup()
    sbar.exec(shell_quote(script) .. " render --format tsv", function(output)
        local lines = split_lines(output)
        local mine_shown = 0
        local review_shown = 0
        local has_mine = false
        local has_review = false

        for _, line in ipairs(lines) do
            local fields = split_tabs(line)
            local section = fields[1] or ""
            local display_name = fields[2] or ""
            local age = fields[3] or ""
            local status = fields[4] or ""
            local title = fields[5] or ""
            local url = fields[6] or ""
            local is_stack = fields[7] == "true"
            local stack_position = fields[8] or ""

            if section == "mine" then
                has_mine = true
                if mine_shown < max_rows_per_section then
                    mine_shown = mine_shown + 1
                    set_row(mine_rows[mine_shown], display_name, age, status, title, url, is_stack, stack_position)
                end
            elseif section == "review" then
                has_review = true
                if review_shown < max_rows_per_section then
                    review_shown = review_shown + 1
                    set_row(review_rows[review_shown], display_name, age, status, title, url, is_stack, stack_position)
                end
            end
        end

        hide_unused(mine_rows, mine_shown + 1)
        hide_unused(review_rows, review_shown + 1)

        set_headers(has_mine, has_review)
        empty_row:set({ drawing = not has_mine and not has_review })
    end)
end

local function refresh_cache()
    prs:set({ icon = { color = colors.blue }, label = { color = colors.blue } })
    sbar.exec(shell_quote(script) .. " refresh >/dev/null 2>&1", function(_)
        render_summary()
        if popup_visible() then
            render_popup()
        end
    end)
end

prs:subscribe({ "forced", "system_woke", "prs_update" }, function(_)
    render_summary()
    if popup_visible() then
        render_popup()
    end
end)

prs:subscribe("routine", function(_)
    refresh_cache()
end)

prs:subscribe("mouse.clicked", function(_)
    render_popup()
    prs:set({ popup = { drawing = "toggle" } })
end)

refresh:subscribe("mouse.clicked", function(_)
    refresh_cache()
end)

prs:subscribe("mouse.exited.global", function(_)
    prs:set({ popup = { drawing = false } })
end)

render_summary()
