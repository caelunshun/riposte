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

    if newData.researchingTech == nil then self.researchingTech = nil end

    self.unlockedTechIDs = {}
    for _, tech in ipairs(newData.unlockedTechIDs or {}) do
        self.unlockedTechIDs[tech] = true
    end
end

function Player:isTechUnlocked(techID)
    return self.unlockedTechIDs[techID] == true
end

function Player:estimateResearchTurns(tech, progress)
    return math.ceil((tech.cost - (progress or 0)) / self.beakerRevenue)
end

return Player
