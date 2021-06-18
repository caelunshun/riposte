-- The main menu.

local dume = require("dume")
local Vector = require("brinevector")

local Image = require("widget/image")
local Container = require("widget/container")
local Navigator = require("widget/navigator")
local Center = require("widget/center")
local Flex = require("widget/flex")
local Clickable = require("widget/clickable")
local Text = require("widget/text")
local StyleModifier = require("widget/style_modifier")
local Fixed = require("widget/fixed")
local Padding = require("widget/padding")

local function build(ui)
    local entries = { "SINGLEPLAYER", "MULTIPLAYER", "OPTIONS" }

    local main = Flex:column()
    main:setMainAlign(dume.Align.Center)
    main:setCrossAlign(dume.Align.Center)
    main:setSpacing(20)

    main.style = {}
    main.style.defaultTextStyle = ui.style.defaultTextStyle
    main.style.defaultTextStyle.weight = dume.FontWeight.Light

    main.style.hovered = {}
    local hoveredText = {
        color = dume.rgb(255, 191, 63)
    }
    setmetatable(hoveredText, hoveredText)
    hoveredText.__index = main.style.defaultTextStyle
    main.style.hovered.defaultTextStyle = hoveredText

    for _, entry in ipairs(entries) do
        local text = Text:new("@size{24}{%entry}", {entry = entry})
        local clickable = Clickable:new(text, function()
            print("Selected " .. entry)
        end)
        main:addFixedChild(StyleModifier:new(clickable))
    end

    local navigator = Navigator:new(
            {
                main = Center:new(Container:new(Padding:new(Fixed:new(main, Vector(300, 200)), 50))),
            },
            "main"
    )

    local root = Image:new("icon/menu", nil, navigator)

    ui:createWindow("mainMenu", Vector(0, 0), Vector(ui.cv:getWidth(), ui.cv:getHeight()), root)
end

return build
