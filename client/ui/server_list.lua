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

local ip = "127.0.0.1"
local port = 19836

local function enterLobby(lobby)
    lobby:rebuild()
    ui:deleteWindow("multiplayerList")
end

local function showError(message)
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

local function checkError(conn)
    if conn:getError() ~= nil then
        showError(conn:getError())
        return true
    end
    return false
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
    local conn = NetworkConnection:new(ip, port)
    checkError(conn)
    conn:sendMessage(json.encode({
        type = "requestGameList"
    }))
    if checkError(conn) then return end

    local response = conn:recvMessage()
    if checkError(conn) then return end
    print(response)
    self.availableGames = json.decode(response)

    self:rebuild()
end

function ServerList:createGame()
    local conn = NetworkConnection:new(ip, port)
    if checkError(conn) then return end
    conn:sendMessage(json.encode({
        type = "createGame",
        info = {
            numHumanPlayers = 1,
            neededHumanPlayers = 2,
            totalPlayers = 7,
        }
    }))
    if checkError(conn) then return end

    return conn
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
                print("joining game")
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
        local conn = self:createGame()
        enterLobby(Lobby:new(nil, conn))
    end))

    return wrapWithMenuBackButton(root)
end

function ServerList:rebuild()
    ui:createWindow("multiplayerList", Vector(0, 0), Vector(cv:getWidth(), cv:getHeight()), Container:new(Padding:new(self:buildRootWidget(), 50)))
end

return ServerList
