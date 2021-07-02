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
local Padding = require("widget/padding")
local Button = require("widget/button")
local Spacer = require("widget/spacer")

local Lobby = require("ui/lobby")

local navigator = nil

local function wrapWithBackButton(widget)
    widget:addFixedChild(Spacer:new(dume.Axis.Vertical, 52))
    widget:addFixedChild(Button:new(Padding:new(Text:new("@size{20}{BACK}"), 10), function()
        navigator:setPage("main")
    end))
    return widget
end

local function build(ui)
    local entries = {
        {
            name = "SINGLEPLAYER",
            onclick = function()
                local bridge = createSingleplayerGame()
                enterGame(bridge)
            end
        },
        {
            name = "MULTIPLAYER",
            onclick = function()
                navigator:setPage("lobby")
            end
        },
        {
            name = "OPTIONS",
            onclick = function()  end
        }
    }

    local main = Flex:column()
    main:setMainAlign(dume.Align.Center)
    main:setCrossAlign(dume.Align.Center)
    main:setSpacing(20)

    for _, entry in ipairs(entries) do
        local text = Text:new("@size{24}{%entry}", {entry = entry.name})
        table.insert(text.classes, "hoverableText")
        local clickable = Clickable:new(text, function()
            print("Selected " .. entry.name)
            entry.onclick()
        end)
        main:addFixedChild(clickable)
    end

    navigator = Navigator:new(
            {
                main = Container:new(Padding:new(main, 50)),
                lobby = Container:new(Padding:new(wrapWithBackButton(Lobby:new():buildRootWidget()), 50)),
            },
            "main"
    )

    local root = Image:new("icon/menu", nil, navigator)

    ui:createWindow("mainMenu", Vector(0, 0), Vector(ui.cv:getWidth(), ui.cv:getHeight()), root, true)
end

return build
