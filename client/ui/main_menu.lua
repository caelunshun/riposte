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

local ServerList = require("ui/server_list")

local build

local isMuted = false

function wrapWithMenuBackButton(widget)
    widget:addFixedChild(Spacer:new(dume.Axis.Vertical, 52))
    widget:addFixedChild(Button:new(Padding:new(Text:new("@size{20}{BACK}"), 10), function()
        ui:deleteWindow("multiplayerList")
        ui:deleteWindow("multiplayerLobby")
        build(ui)
    end))
    return widget
end

build = function (ui)
    local entries = {
        {
            name = "SINGLEPLAYER",
            onclick = function()
                local bridge = createServer().bridge
                enterGame(bridge, false)
            end
        },
        {
            name = "MULTIPLAYER",
            onclick = function()
                ServerList:new():rebuild()
                ui:deleteWindow("mainMenu")
            end
        },
        --{
            --name = "OPTIONS",
            --onclick = function()  end
        --},
        {
            name = "TOGGLE MUTE",
            onclick = function()
                if isMuted then
                    setGlobalVolume(1)
                else
                    setGlobalVolume(0)
                end
                isMuted = not isMuted
            end
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

    local root = Container:new(Padding:new(main, 50))

    ui:createWindow("mainMenu", dume.FillScreen, root, 1)

    ui:createWindow("menuBackgroundImage", dume.FillScreen, Image:new("icon/menu", nil, nil, true), -1)
end

return build
