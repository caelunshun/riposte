local Player = {}

function Player:new(data)
    data.civ = registry.civs[data.civID]
    if data.civ == nil then print("received invalid civ " .. data.civID .. "!") end

    setmetatable(data, self)
    self.__index =self
    return data
end

function Player:updateData(newData)
    for k, v in pairs(newData) do
        self[k] = v
    end
end

function Player:estimateResearchTurns(tech)
    return math.ceil(tech.cost / self.beakerRevenue)
end

return Player
