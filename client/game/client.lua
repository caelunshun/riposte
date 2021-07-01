-- Handles packets.
local Client = {}

local pb = require("pb")
local protoc = require("protoc")

local protoFile = io.open("proto/riposte.proto")
local protoData = protoFile:read("*all")
protoFile:close()
protoc:load(protoData)

function Client:new(game, bridge)
    local o = {
        game = game,
        bridge = bridge,

        nextRequestID = 1,
        responseCallbacks = {},
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

-- Requests that the server compute the shortest path the given unit
-- can take between two points.
--
-- Must be called from within a coroutine. When the server responds, this
-- function returns the computed path, which may be nil if no possible path
-- exists.
function Client:requestComputePath(from, to, unitKindID)
    local thread = coroutine.running()
    self:sendPacket("computePath", {
        from = { x = from.x, y = from.y },
        to = { x = to.x, y = to.y },
        unitKindID = unitKindID,
        requestID = requestID,
    }, function(packet)
        callSafe(thread, packet)
    end)

    local packet = coroutine.yield()
    return packet.path
end

-- Moves the given units along a path.
-- Returns whether moving was successful.
-- Must be called from within a coroutine.
function Client:moveUnitsAlongPath(units, path)
    local unitIDs = {}
    for _, unit in ipairs(units) do
        unitIDs[#unitIDs + 1] = unit.id
    end
    assert(path.positions ~= nil)

    local thread = coroutine.running()
    self:sendPacket("moveUnits", {
        unitIDs = unitIDs,
        pathToFollow = path,
    }, function(response)
        callSafe(thread, response)
    end)

    local response = coroutine.yield()
    assert(response.success ~= nil)
    return response.success
end

-- Requests a list of the possible build tasks for the given city.
-- Must be called from a coroutine.
function Client:getPossibleBuildTasks(city)
    local thread = coroutine.running()
    self:sendPacket("getBuildTasks", {
        cityID = city.id,
    }, function(response)
        callSafe(thread, response.tasks)
    end)
    return coroutine.yield()
end

-- Requests a list of the possible researchable techs for our player.
-- Must be called from a coroutine.
function Client:getPossibleTechs()
    local thread = coroutine.running()
    self:sendPacket("getPossibleTechs", {}, function(response)
        callSafe(thread, response.techs)
    end)
    return coroutine.yield()
end

function Client:setResearch(tech)
    self:sendPacket("setResearch", { techID = tech.name })
end

function Client:setCityBuildTask(city, buildTaskKind)
    self:sendPacket("setCityBuildTask", {
        cityID = city.id,
        task = buildTaskKind,
    })
end

function Client:setEconomySettings(beakerPercent)
    self:sendPacket("setEconomySettings", {
        beakerPercent = beakerPercent,
    })
end

function Client:doUnitAction(unit, action)
    self:sendPacket("doUnitAction", {
        unitID = unit.id,
        action = action,
    })
end

function Client:setWorkerTask(workerUnit, task)
    self:sendPacket("setWorkerTask", {
        workerID = workerUnit.id,
        task = task,
    })
end

function Client:endTurn()
    self:sendPacket("endTurn", {})
end

function Client:sendPacket(packetKind, packet, callback)
    print("Sending packet: " .. packetKind)
    local requestID = self.nextRequestID
    self.nextRequestID = self.nextRequestID + 1
    local p = {
        [packetKind] = packet,
        requestID = requestID,
    }
    local data = pb.encode("AnyClient", p)
    self.bridge:sendPacket(data)

    if callback ~= nil then
        self.responseCallbacks[requestID] = callback
    end
end

function Client:handleReceivedPackets()
    local packet = self.bridge:pollReceivedPacket()
    while packet ~= nil do
        local decodedPacket = pb.decode("AnyServer", packet)

        local packetType = nil
        for k, v in pairs(decodedPacket) do
            if k ~= "requestID" then
                packetType = k
                print("Received packet: " .. packetType)
                break
            end
        end

        self:handlePacket(decodedPacket)

        local callback = self.responseCallbacks[decodedPacket.requestID]
        if callback ~= nil then
            callback(decodedPacket[packetType])
            self.responseCallbacks[decodedPacket.requestID] = nil
        end

        packet = self.bridge:pollReceivedPacket()
    end
end

function Client:handlePacket(packet)
    -- Packet is an AnyServer message (a union)
    if packet.updateGlobalData ~= nil then
        self:handleUpdateGlobalData(packet.updateGlobalData)
    elseif packet.updateMap ~= nil then
        self:handleUpdateMap(packet.updateMap)
    elseif packet.updateVisibility ~= nil then
        self:handleUpdateVisibility(packet.updateVisibility)
    elseif packet.updateTile ~= nil then
        self:handleUpdateTile(packet.updateTile)
    elseif packet.updateUnit ~= nil then
        self:handleUpdateUnit(packet.updateUnit)
    elseif packet.updateCity ~= nil then
        self:handleUpdateCity(packet.updateCity)
    elseif packet.updatePlayer ~= nil then
        self:handleUpdatePlayer(packet.updatePlayer)
    elseif packet.deleteUnit ~= nil then
        self:handleDeleteUnit(packet.deleteUnit)
    end
end

function Client:handleUpdateGlobalData(packet)
    for _, playerdata in ipairs(packet.players) do
        self.game:updatePlayer(playerdata)
    end

    self.game:setTurn(packet.turn)
    self.game:setEra(packet.era)
    self.game.thePlayer = self.game.players[packet.playerID]
    if self.game.thePlayer == nil then
        error("invalid thePlayer ID")
    end

    self.game.eventBus:trigger("globalDataUpdated", nil)
end

function Client:handleUpdateMap(packet)
    self.game.tiles = packet.tiles
    self.game.mapWidth = packet.width
    self.game.mapHeight = packet.height
end

function Client:handleUpdateVisibility(packet)
    self.game.visibility = packet.visibility
end

function Client:handleUpdateTile(packet)
    self.game.tiles[packet.x + packet.y * self.game.mapWidth + 1] = packet.tile
end

function Client:handleUpdateUnit(packet)
    self.game:addUnit(packet)
end

function Client:handleUpdateCity(packet)
    self.game:addCity(packet)
end

function Client:handleUpdatePlayer(packet)
    self.game:updateThePlayer(packet)
end

function Client:handleDeleteUnit(packet)
    self.game:deleteUnit(packet.unitID)
end

return Client
