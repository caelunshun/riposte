-- The multiplayer lobby, which is displayed before a game has started.
-- However, we are connected to a server.

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
local Table = require("widget/table")
local Spacer = require("widget/spacer")
local Empty = require("widget/empty")
local Divider = require("widget/divider")

local UiUtils = require("ui/utils")
local Client = require("game/client")

local Lobby = {}

-- Creates a new lobby.
--
-- serverBridge is the Bridge with the server hosting the lobby.
--
-- lobbyServerConn is our connection with the lobby server, which is not null
-- only if we're the game host. It's used to accept new connections and to update
-- game info in the server list.
function Lobby:new(serverBridge, lobbyServerConn)
    local o = {
        serverBridge = serverBridge,
        lobbyServerConn = lobbyServerConn,
        isTheHost = (lobbyServerConn ~= nil),
        client = Client:new(nil, serverBridge),
    }

    o.client:sendClientInfo("caelunshun") -- TODO: don't hardcode username

    setmetatable(o, self)
    self.__index = self
    return o
end

function Lobby:buildRootWidget()
    local root = Flex:column()
    root:setCrossAlign(dume.Align.Center)

    root:addFixedChild(Text:new("@size{24}{Game Lobby}"))

    local subtitle
    if self.isTheHost then
        subtitle = "Waiting for players to join..."
    else
        subtitle = "Waiting for the host to start the game..."
    end
    root:addFixedChild(Text:new(subtitle))

    root:addFixedChild(Spacer:new(dume.Axis.Vertical, 20))

    -- Players table
    root:addFixedChild(Divider:new(2))
    root:addFixedChild(Text:new("@size{20}{Players}"), {}, {alignH = dume.Align.Center})

    local rows = {}
    -- title row
    rows[1] = {
        username = Text:new("Username"),
        civ = Text:new("Civilization"),
        leader = Text:new("Leader"),
    }

    for _, player in ipairs(self.players or {}) do
        if player.exists then
            local civ = registry.civs[player.civID]

            local usernameText = "%username"
            if player.isAdmin then
                usernameText = "@icon{crown} " .. usernameText
            end

            rows[#rows + 1] = {
                username = Text:new(usernameText, {username=player.username}),
                civ = Text:new("@color{%color}{%name}", {name=civ.name, color=dumeColorToString(civ.color)}),
                leader = Text:new(player.leaderName),
            }
        end
    end

    root:addFixedChild(Table:new(
            {
                "username",
                "civ",
                "leader",
            },
            rows
    ))

    return UiUtils.createWindowContainer(root)
end

function Lobby:rebuild()
    ui:createWindow("multiplayerLobby", Vector(0, 0), Vector(cv:getWidth(), cv:getHeight()), self:buildRootWidget())
end

function Lobby:tick()
    self.client:handleReceivedPacketsWithHandler(function(packet)
        self:handlePacket(packet)
    end)
end

function Lobby:handlePacket(packet)
    if packet.serverInfo ~= nil then
        self:handleServerInfo(packet.serverInfo)
    end
end

function Lobby:handleServerInfo(packet)
    self.players = packet.players

    for _, player in ipairs(packet.players) do
        if player.id == packet.thePlayerID then
            self.thePlayer = player
        end
    end

    self:rebuild()
end

return Lobby
