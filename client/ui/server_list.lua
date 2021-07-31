-- The multiplayer server list.
local ServerList = {}

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

local Lobby = require("ui/lobby")

local json = require("lunajson")

lobbyIP = "127.0.0.1"
lobbyPort = 19836

local function enterLobby(lobby)
    lobby:rebuild()
    ui:deleteWindow("multiplayerList")
    _G.enterLobby(lobby)
end

function showError(message)
    local root = Flex:column()
    root:addFixedChild(Text:new("@size{20}{%message}", {message=message}))
    root:addFixedChild(Button:new(Text:new("OK"), function()
        ui:deleteWindow("errorDialogue")
    end))
    root:setCrossAlign(dume.Align.Center)
    local size = Vector(300, 100)
    local container = UiUtils.createWindowContainer(root)
    ui:createWindow("errorDialogue", Vector(cv:getWidth() / 2 - size.x / 2, cv:getHeight() / 2 - size.y / 2), size, container)
end

local function joinGame(gameID)
    networking:connectAsync(lobbyIP, lobbyPort, function(connHandle, error)
        if error ~= nil then
            showError(error)
            return
        end

        networking:sendMessage(connHandle, json.encode({
            type = "joinGame",
            id = gameID,
        }))

        networking:convertToBridgeAsync(connHandle, function(bridge)
            local lobby = Lobby:new({bridge = bridge}, nil)
            enterLobby(lobby)
        end)
    end)
end

function ServerList:new()
    local o = {
        availableGames = {}
    }
    setmetatable(o, self)
    self.__index = self

    o:updateAvailableGames()

    return o
end

function ServerList:updateAvailableGames()
    networking:connectAsync(lobbyIP, lobbyPort, function(connHandle, error)
        if error ~= nil then
            showError(error)
            return
        end

        -- Send RequestGameList message
        networking:sendMessage(connHandle, json.encode({
            type = "requestGameList"
        }))

        -- Wait for response
        networking:recvMessageAsync(connHandle, function(messageJSON, error)
            if error ~= nil then
                showError(error)
                return
            end

            self.availableGames = json.decode(messageJSON)
            self:rebuild()
        end)
    end)
end

function ServerList:createGame(callback)
    networking:connectAsync(lobbyIP, lobbyPort, function(connHandle, error)
        if error ~= nil then
            showError(error)
            return
        end

        networking:sendMessage(connHandle, json.encode({
            type = "createGame",
            info = {
                numHumanPlayers = 1,
                neededHumanPlayers = 2,
                totalPlayers = 7,
            }
        }))

        callback(connHandle)
    end)
end

function ServerList:buildRootWidget()
    local root = Flex:column()
    root:setMainAlign(dume.Align.Center)
    root:setCrossAlign(dume.Align.Center)

    root:addFixedChild(Text:new("@size{20}{Multiplayer Games}"))

    root:addFixedChild(Text:new("Available: " .. #self.availableGames))

    local rows = {}

    -- Header row
    rows[1] = {
        currentPlayers = Text:new("Players Waiting"),
        neededPlayers = Text:new("Players Needed"),
        totalPlayers = Text:new("Total Players"),
        join = Empty:new(),
    }

    for _, game in ipairs(self.availableGames) do
        rows[#rows + 1] = {
            currentPlayers = Text:new(tostring(game.numHumanPlayers)),
            neededPlayers = Text:new(tostring(game.neededHumanPlayers)),
            totalPlayers = Text:new(tostring(game.totalPlayers)),
            join = Button:new(Text:new("Join"), function()
                joinGame(game.id)
            end)
        }
    end

    local table = Table:new(
            {
                "currentPlayers",
                "neededPlayers",
                "totalPlayers",
                "join",
            },
            rows
    )
    table.minSize = Vector(100, 200)
    root:addFixedChild(table)

    root:addFixedChild(Spacer:new(dume.Axis.Vertical, 20))

    root:addFixedChild(Button:new(Text:new("@size{18}{Create Game}"), function()
        self:createGame(function(conn)
            local server = createServer()
            enterLobby(Lobby:new(server, conn))
        end)
    end))

    return wrapWithMenuBackButton(root)
end

function ServerList:rebuild()
    ui:createWindow("multiplayerList", Vector(0, 0), Vector(cv:getWidth(), cv:getHeight()), Container:new(Padding:new(self:buildRootWidget(), 50)))
end

return ServerList
