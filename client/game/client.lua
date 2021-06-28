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

        nextPathRequestID = 0,
        computedPathCallbacks = {},
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

-- `callback` is invoked when the server responds
-- with the computed path.
-- `callback` takes a `Path` parameter.
-- The parameter may be nil if no valid path could be found.
function Client:requestComputePath(callback, from, to, unitKindID)
    local requestID = self.nextPathRequestID
    self.nextPathRequestID = self.nextPathRequestID + 1
    self:sendPacket("computePath", {
        from = { x = from.x, y = from.y },
        to = { x = to.x, y = to.y },
        unitKindID = unitKindID,
        requestID = requestID,
    })
    self.computedPathCallbacks[requestID] = callback
end

function Client:sendPacket(packetKind, packet)
    local p = {
        [packetKind] = packet,
    }
    local data = pb.encode("AnyClient", p)
    self.bridge:sendPacket(data)
end

function Client:handleReceivedPackets()
    local packet = self.bridge:pollReceivedPacket()
    while packet ~= nil do
        local decodedPacket = pb.decode("AnyServer", packet)
        for k, _ in pairs(decodedPacket) do print("Received packet: " .. k) end
        self:handlePacket(decodedPacket)
        packet = self.bridge:pollReceivedPacket()
    end
end

function Client:handlePacket(packet)
    -- Packet is an AnyServer message (a union)
    if packet.updateGlobalData ~= nil then
        self:handleUpdateGlobalData(packet.updateGlobalData)
    elseif packet.updateMap ~= nil then
        self:handleUpdateMap(packet.updateMap)
    elseif packet.updateUnit ~= nil then
        self:handleUpdateUnit(packet.updateUnit)
    elseif packet.updateCity ~= nil then
        self:handleUpdateCity(packet.updateCity)
    elseif packet.pathComputed ~= nil then
        self:handleComputedPath(packet.pathComputed)
    else
        error("unhandled packet")
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
    self.game.visibility = packet.visibility
end

function Client:handleUpdateUnit(packet)
    self.game:addUnit(packet)
end

function Client:handleUpdateCity(packet)
    self.game:addCity(packet)
end

function Client:handleComputedPath(packet)
    local callback = self.computedPathCallbacks[packet.requestID]
    if callback ~= nil then
        callback(packet.path)
        self.computedPathCallbacks[packet.requestID] = nil
    end
end

return Client
