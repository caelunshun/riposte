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

local UiUtils = require("ui/utils")

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
    }

    setmetatable(o, self)
    self.__index = self
    return o
end

function Lobby:buildRootWidget()
    local root = Flex:column()
    root:setCrossAlign(dume.Align.Center)

    root:addFixedChild(Text:new("@size{24}{Multiplayer Lobby}"))

    root:addFixedChild(Spacer:new(dume.Axis.Vertical, 20))

    local subtitle
    if self.isTheHost then
        subtitle = "Waiting for players to join..."
    else
        subtitle = "Waiting for the host to start the game..."
    end
    root:addFixedChild(Text:new(subtitle))

    return UiUtils.createWindowContainer(root)
end

function Lobby:rebuild()
    ui:createWindow("multiplayerLobby", Vector(0, 0), Vector(cv:getWidth(), cv:getHeight()), self:buildRootWidget())
end

return Lobby
