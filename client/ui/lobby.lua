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
local Tooltip = require("widget/tooltip")

local UiUtils = require("ui/utils")
local Client = require("game/client")

local Lobby = {}

local json = require("lunajson")

local function defaultSettings()
    return {
        mapWidth = 16,
        mapHeight = 16,
        numHumanPlayers = 2,
        numAIPlayers = 0,
    }
end

-- Creates a new lobby.
--
-- server is the Bridge with the server hosting the lobby.
--
-- lobbyServerConn is our connection with the lobby server, which is not null
-- only if we're the game host. It's used to accept new connections and to update
-- game info in the server list.
function Lobby:new(server, lobbyServerConn)
    local o = {
        server = server,
        lobbyServerConn = lobbyServerConn,
        isTheHost = (lobbyServerConn ~= nil),
        client = Client:new(nil, server.bridge),
        settings = defaultSettings(),
        listeningForConnections = false,
    }

    o.client:sendClientInfo("caelunshun") -- TODO: don't hardcode username

    setmetatable(o, self)
    self.__index = self

    o:waitForNextClient()
    o:sendGameOptions()

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

    root:addFixedChild(Divider:new(2))

    local bottom = Flex:row()
    bottom:setMainAlign(dume.Align.Center)

    -- Players table
    local players = Flex:column()
    players:addFixedChild(Text:new("@size{20}{Players}"), {}, {alignH = dume.Align.Center})

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

    players:addFixedChild(Table:new(
            {
                "username",
                "civ",
                "leader",
            },
            rows
    ))
    bottom:addFlexChild(players, 1)

    local settings = Flex:column()
    settings:addFixedChild(Text:new("@size{20}{Settings}"), {}, {alignH = dume.Align.Center})

    settings:addFixedChild(Divider:new(1))

    settings:addFixedChild(Text:new("Map Width: " .. tostring(self.settings.mapWidth) .. " tiles"))
    settings:addFixedChild(Text:new("Map Height: " .. tostring(self.settings.mapHeight) .. " tiles"))

    settings:addFixedChild(Spacer:new(dume.Axis.Vertical, 20))
    settings:addFixedChild(Divider:new(1))
    settings:addFixedChild(Spacer:new(dume.Axis.Vertical, 20))

    settings:addFixedChild(Text:new(tostring(self.settings.numHumanPlayers) .. " Human Players"))
    settings:addFixedChild(Text:new(tostring(self.settings.numAIPlayers) .. " AI Players"))
    settings:addFixedChild(Text:new(tostring(self.settings.numHumanPlayers + self.settings.numAIPlayers) .. " Total Players"))

    bottom:addFlexChild(settings, 1)

    root:addFlexChild(bottom, 1)

    -- Footer
    local footer = Flex:row()

    if self.isTheHost then
        local canStartGame = self:isReadyToStart()
        local startGame = Button:new(Padding:new(Text:new("@size{20}{Start Game}"), 10), function()
            if canStartGame then
                self.client:adminStartGame()
                enterGame(self.server.bridge, true)
            end
        end)

        if not canStartGame then
            startGame = Tooltip:new(startGame, Container:new(Padding:new(Text:new("@color{rgb(200,40,70)}{The game cannot start until enough human players have joined.}"), 10)))
        end

        footer:addFixedChild(startGame)
    end

    root:addFixedChild(footer)

    return UiUtils.createWindowContainer(root)
end

function Lobby:rebuild()
    ui:createWindow("multiplayerLobby", Vector(0, 0), Vector(cv:getWidth(), cv:getHeight()), self:buildRootWidget(), 1)
end

function Lobby:tick()
    self.client:handleReceivedPacketsWithHandler(function(packet)
        return self:handlePacket(packet)
    end)
end

function Lobby:handlePacket(packet)
    if packet.serverInfo ~= nil then
        self:handleServerInfo(packet.serverInfo)
    elseif packet.startGame ~= nil then
        return self:handleStartGame(packet.startGame)
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

function Lobby:handleStartGame(packet)
    enterGame(self.server.bridge, true)
    -- stop handling packets so that game data packets are handled by the game client instead of the lobby client
    return true
end

function Lobby:getNumHumanPlayers()
    local count = 0
    for _, player in ipairs(self.players or {}) do
        if player.isHuman and player.exists then
            count = count + 1
        end
    end
    return count
end

function Lobby:isReadyToStart()
    return self:getNumHumanPlayers() == self.settings.numHumanPlayers
end

function Lobby:sendGameOptions()
    self.client:sendGameOptions(self.settings)
end

function Lobby:waitForNextClient()
    if self.settings.numHumanPlayers == self:getNumHumanPlayers() then
        self.listeningForConnections = false
        return
    end

    if self.lobbyServerConn == nil then return end -- not the game host

    networking:recvMessageAsync(self.lobbyServerConn, function(messageJSON, error)
        if error ~= nil then
            showError(error)
            return
        end

        local message = json.decode(messageJSON)
        print(messageJSON)

        -- Create proxied connection with new client.
        networking:connectAsync(lobbyIP, lobbyPort, function(connHandle, error)
            if error ~= nil then
                showError(error)
            end

            networking:sendMessage(connHandle, json.encode({
                type = "proxyWithClient",
                client_id = message.client_id,
            }))

            networking:convertToBridgeAsync(connHandle, function(bridge)
                addServerConnection(self.server.newConnections, bridge)
            end)
        end)

        self:waitForNextClient()
    end)
    self.listeningForConnections = true
end

return Lobby
